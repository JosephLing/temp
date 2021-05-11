module HttpResponses
    extend ActiveSupport::Concern

    def json_ok(obj, response)
        render :status => response, :json => obj
    end
end