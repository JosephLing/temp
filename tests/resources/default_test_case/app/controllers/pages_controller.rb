class PagesController < ApplicationController
    include PageHelper

    before_action :get_page_number

    def get_page_number
        @page_index = params[:index]
    end

    def index
        return unless user_details
        foobar(params)
        json_ok(blog_category.where(page: @page_index).to_json, 200)
    end

    def show
        @data = 1
    end

    private 

    def user_details
        User.find(params[:user_id]).where(page: @page_index)
    end
    
    # params -> []
    # second pass knowing that details = params 
    # params -> [:cat]
    def foobar(details)

        details[:cat]
    end
end