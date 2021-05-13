module ErrorHandling
    extend ActiveSupport::Concern
    
    included do
        around_action :catch_exceptions
    end

    def catch_exceptions
        begin
          yield
        rescue Exception => e
          unless @catch_all_exceptions
            raise e
          else
            logger.error("ERROR caught in application controller")
            logger.error(e.backtrace.join("\n"))
            json(500, e.message)
          end
        end
    end

end