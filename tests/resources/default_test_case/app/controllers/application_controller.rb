class ApplicationController < ActionController::API 
    include HttpResponses

    before_action :auth_check

    def auth_check
        return unless params[:auth_token] == 1
    end
end