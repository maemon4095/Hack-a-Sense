require "fileutils"

def category_page_content(category)
    return <<~TEXT
    ---
    layout: category
    parmalink: "/categories/#{Jekyll::Utils.slugify(category)}"
    category: #{category}
    ---
    TEXT
end

Jekyll::Hooks.register :site, :post_write do |site|
    config = site.config
    source = File.expand_path(config["source"])
    path = config["collections"]["categories"]["directory"]
    dir = File.join(source, path)
    if File.exists?(dir) then
        p "delete #{dir}"
        FileUtils.rm_r(dir)
    end
    p "create #{dir}"
    Dir.mkdir(dir)

    site.tags.keys.each do |tag|
        file = File.join(dir, "#{tag}.html") 
        File.write(file, category_page_content(tag))
        p "generated: #{file}"
    end
end