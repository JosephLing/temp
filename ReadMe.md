# Installation

1. Download rust up from: https://www.rust-lang.org/tools/install
2. close and reopen terminal 
3. Run `cargo check` and it should install all the packages and it will check whether or not it can compile the code for you
4. install https://marketplace.visualstudio.com/items?itemName=matklad.rust-analyzer (note: need to click on download now pop up for it to work)
5. Run `cargo run tests/resources/default_test_case` 

# Top tips for working on the project:
- checking out the AST/strange ruby syntax: https://lib-ruby-parser.github.io/wasm-bindings/
- write lots of unit tests for strange edge cases
- `cargo test -- <mod name of the tests you want to run>` e.g. `cargo test -- routes_parsing`

# tasks:
Views:
- failing jbuilder tests
- jbuilder string vec to json 
- jb parsing (in theory if jbuilder works should be a lot easier and simpler to do...)
- handling loading in partials
- (advanced) - parse `schema.rb` to add guess work types to the json objects

Routes:
- routes parse to `routes.rb` instead of parsing the output of `bundle exec rails r routes > test.routes`

Testing:
- integeration tests
    - does our example code actually boot a rails app?
    - can we have our example code being an actual test? - or at least add tests around the look up code
    - controllers in modules - do we handle parsing them properly
    - recursive method

File type
- module actions
- custom actions - how are we going to handle these
- concerns `included`
- caching - does this make things faster on avergage??
    - build up object of Application controllers so they don't have to be cached each time

Method details parser:
- parse method calls better e.g. `User.where().foobar()` -> `["where", "foobar"]` and in this case we just want `[""]`
- headers and cookies
- params.keys?
- what does the method return? last statement in the body of the method
    - this shouldn't be that important except for the edge case of:
    ```ruby
    class PagesController < ApplicationController
        def index
            extract_params(handle_params(params))
        end

        def handle_params(p)
            return error unless p[:magic_token]
            p
        end

        def extract_params(p)
            p[:id]
        end
    end

    ```
    However this does also have the edge case we don't currently handle of params being passed in as an argument...

Open api / swagger
- format output into common format

# future work:
Method details stores a hash map of every time a local varaible is accessed, therefore we can do:
```rust
for (k, v) in &local_varaibles {
        if *v == 0 {
            println!("local varaible '{}' is never used", k);
        }
    }
```
(side note: this pretty much comes for free as the parser handles this for us)

# Example:

This currently works, including basic .jbuilder support. Parsing routes file is done through parsing the output of `bundle exec rails r routes > test.routes` though for now.

```ruby

    class ApplicationController < ActionController::API
        include HttpResponses

        before_action: auth_check

        def auth_check
            return unless params[:auth_token] == 1
        end
    end

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

    module PageHelper
        def blog_category
            Blogs.find(params[:cat])
        end
    end

    module HttpResponses
        extend ActiveSupport::Concern

        def json_ok(obj, response)
            render :status => response, :json => obj
        end
    end

    # routes.rb

    get 'pages/index' => 'page#index'
    get 'blog/:cat/pages/index' => 'page#index'
```

By running our script to get the output:
- pages/index takes auth_token, index, user_id,cat
- blog/:cat/pages/index takes auth_token, index, user_id,cat
