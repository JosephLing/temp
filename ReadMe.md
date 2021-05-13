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
- parasing params
    - index
    - send 
    - test cases for all the edge cases
- hooking up the controller includes and parent class together nicely 
    - allowing a second parse to know about all the functions/methods that should be available

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

# Goal:

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




- controller stuff 99%
- routes - I'll sort 

pages/index

call index - what methods does this call? do any these methods use params?

method_details.methods_calls and .params

Stretch goal: foobar(params)

pages/index
{
    id,
    title,
    description,
    ?read_count
}
