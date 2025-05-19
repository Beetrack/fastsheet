extern crate libc;
extern crate calamine;

use std::ffi::{CString, CStr};
use libc::{c_int, c_void, c_char, c_double, uintptr_t};
use calamine::{Sheets, DataType};

// VALUE (pointer to a ruby object)
type Value = uintptr_t;

// Some ruby constant values
const NIL: usize   = 0x08;
const TRUE: usize  = 0x14;
const FALSE: usize = 0x00;

// Load some Ruby API functions
extern "C" {
    // Object class
    static rb_cObject: Value;

    // Modules and classes
    fn rb_define_module(name: *const c_char) -> Value;
    fn rb_define_class_under(outer: Value, name: *const c_char, superclass: Value) -> Value;
    fn rb_define_method(class: Value, name: *const c_char, method: *const c_void, argc: c_int) -> Value;

    // Set instance variables
    fn rb_iv_set(object: Value, name: *const c_char, value: Value) -> Value;

    // Array
    fn rb_ary_new() -> Value;
    fn rb_ary_push(array: Value, elem: Value) -> Value;

    // C data to Ruby
    fn rb_int2big(num: c_int) -> Value;
    fn rb_float_new(num: c_double) -> Value;
    fn rb_utf8_str_new_cstr(str: *const c_char) -> Value;

    // Ruby string to C string
    fn rb_string_value_cstr(str: *const Value) -> *const c_char;
}

// C string from Rust string
fn cstr(string: &str) -> CString {
    CString::new(string).unwrap_or_else(|_| CString::new("").unwrap())
}

// Rust string from Ruby string
fn rstr(string: Value) -> Result<String, std::str::Utf8Error> {
    unsafe {
        let s = rb_string_value_cstr(&string);
        CStr::from_ptr(s).to_str().map(|s| s.to_string())
    }
}

// Read the sheet
unsafe fn read(this: Value, rb_file_name: Value) -> Value {
    let file_name = match rstr(rb_file_name) {
        Ok(name) => name,
        Err(_) => return NIL,
    };

    let mut document = match Sheets::open(&file_name) {  // Added mut here
        Ok(doc) => doc,
        Err(_) => return NIL,
    };

    let sheet = match document.worksheet_range_by_index(0) {
        Ok(s) => s,
        Err(_) => return NIL,
    };

    let rows = rb_ary_new();

    for row in sheet.rows() {
        let new_row = rb_ary_new();

        for c in row.iter() {
            let value = match c {  // Removed * dereference
                DataType::Error(_) => NIL,
                DataType::Empty => NIL,
                DataType::Float(f) => rb_float_new(*f as c_double),  // Added * dereference
                DataType::Int(i) => rb_int2big(*i as c_int),  // Added * dereference
                DataType::Bool(b) => if *b { TRUE } else { FALSE },  // Added * dereference
                DataType::String(s) => {
                    let st = s.trim();
                    if st.is_empty() {
                        NIL
                    } else {
                        rb_utf8_str_new_cstr(cstr(st).as_ptr())
                    }
                }
            };
            rb_ary_push(new_row, value);
        }

        rb_ary_push(rows, new_row);
    }

    // Set instance variables
    rb_iv_set(
        this,
        cstr("@width").as_ptr(),
        rb_int2big(sheet.width() as i32)
    );

    rb_iv_set(
        this,
        cstr("@height").as_ptr(),
        rb_int2big(sheet.height() as i32)
    );

    rb_iv_set(
        this,
        cstr("@rows").as_ptr(),
        rows
    );

    this
}

#[no_mangle]
pub unsafe extern "C" fn Init_libfastsheet() {
    let fastsheet = rb_define_module(cstr("Fastsheet").as_ptr());
    let sheet = rb_define_class_under(fastsheet, cstr("Sheet").as_ptr(), rb_cObject);

    rb_define_method(
        sheet,
        cstr("read!").as_ptr(),
        read as *const c_void,
        1 as c_int
    );
}

