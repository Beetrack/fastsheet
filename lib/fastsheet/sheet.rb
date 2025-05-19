module Fastsheet
  class Sheet
    attr_reader :file_name,
                :rows, :header,
                :width, :height

    def initialize(file_name, options = {})
      raise ArgumentError, "File not found: #{file_name}" unless File.exist?(file_name)
      
      begin
        read!(file_name)
        @header = @rows.shift if options[:header]
      rescue => e
        raise "Failed to read sheet: #{e.message}"
      end
    end

    def row(n) = @rows[n]

    def each_row(&block)
      return @rows.each unless block
      @rows.each(&block)
    end

    def column(n) = @rows.map { _1[n] }

    def columns = (0...@width).map { |i| column(i) }

    def each_column(&block)
      return columns.each unless block
      columns.each(&block)
    end
  end
end
