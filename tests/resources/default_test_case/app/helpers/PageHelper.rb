module PageHelper
    def blog_category
        Blogs.find(params[:cat])
    end
end

