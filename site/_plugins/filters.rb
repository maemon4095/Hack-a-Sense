module Jekyll
    module AdditionalFilters
        def escape_path(input)
            Jekyll::URL.escape_path(Jekyll::Utils.slugify(input))
        end
        def unescape_path(input)
            Jekyll::URL.unescape_path(input)
        end
        module_function :escape_path
        public :escape_path
        module_function :unescape_path
        public :unescape_path
    end
end
  
Liquid::Template.register_filter(Jekyll::AdditionalFilters)