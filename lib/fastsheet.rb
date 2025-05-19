require 'fiddle'

module Fastsheet
  # Use pattern matching for platform detection
  LIB_EXT = case RUBY_PLATFORM
    in /win32/ then 'dll'
    in /darwin/ then 'dylib'
    else 'so'
  end

  def self.load_native_extension
    lib_path = File.expand_path("../../ext/fastsheet/target/release/libfastsheet.#{LIB_EXT}", __FILE__)
    
    # Ensure the library exists
    unless File.exist?(lib_path)
      raise "Native library not found at #{lib_path}. Please run 'cargo build --release' in ext/fastsheet"
    end

    # Try to load the library
    begin
      lib = Fiddle.dlopen(lib_path)
      
      # Verify the initialization function exists
      unless lib['Init_libfastsheet']
        raise "Init_libfastsheet function not found in the native library"
      end
      
      # Initialize the library
      Fiddle::Function.new(lib['Init_libfastsheet'], [], Fiddle::TYPE_VOIDP).call
      true
    rescue Fiddle::DLError => e
      raise "Failed to load native library: #{e.message}. Please ensure all dependencies are installed."
    rescue => e
      raise "Unexpected error loading native extension: #{e.class} - #{e.message}"
    end
  end

  # Load the native extension
  load_native_extension
rescue => e
  $stderr.puts "Error loading Fastsheet: #{e.message}"
  $stderr.puts e.backtrace
  raise
end

require 'fastsheet/sheet'
