class PagesController < ApplicationController
    before_action :get_page_number

    def get_page_number
        @page_index = params[:index]
    end

    def index
        return unless user_details
        json_ok(blog_category.where(page: @page_index).to_json, 200)
    end

    private 

    def user_details
        User.find(params[:user_id]).where(page: @page_index)
    end
end