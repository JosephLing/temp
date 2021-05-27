/**
Unsupported routes configuration:

1. resolve (https://guides.rubyonrails.org/routing.html#singular-resources)

```ruby
resource :geocoder
resolve('Geocoder') { [:geocoder] }
```


*/
use crate::types::AppData;
use convert_case::{Case, Casing};
use std::collections::HashSet;
use std::collections::VecDeque;
use std::str::FromStr;

use lib_ruby_parser::Node;
#[derive(Debug, PartialEq)]
pub enum RequestMethod {
    Get,
    Post,
    Delete,
    Put,
    Patch,
    Options,
}

impl FromStr for RequestMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "GET" => RequestMethod::Get,
            "POST" => RequestMethod::Post,
            "DELETE" => RequestMethod::Delete,
            "PUT" => RequestMethod::Put,
            "PATCH" => RequestMethod::Patch,
            "OPTIONS" => RequestMethod::Options,
            _ => return Err(format!("unknown Request method '{}'", &(*s).to_owned())),
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Request {
    pub method: RequestMethod,
    pub prefix: String,
    pub uri: String,
    pub controller: String,
    pub action: String,
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {}", self.method, self.uri)
    }
}

impl Request {
    pub fn get_params(&self, app_data: &AppData) -> Result<HashSet<String>, String> {
        if let Some(controller) = app_data
            .controllers
            .get(&self.controller.to_case(Case::Pascal))
        {
            let mut params: HashSet<String>;
            // handle action
            if let Some(method) = controller.get_method_by_name(&self.action, app_data) {
                params = controller.get_method_params(&method, app_data);
            } else {
                return Err(format!(
                    "ERROR: action {} not found in controller {} for request {}",
                    self.action,
                    &self.controller.to_case(Case::Pascal),
                    self.uri
                ));
            }
            // handle before/after/rescue
            for (_, action_name) in &controller.actions {
                if let Some(method) = controller.get_method_by_name(action_name, app_data) {
                    params.extend(controller.get_method_params(&method, app_data));
                } else {
                    return Err(format!(
                        "ERROR: action {} not found in controller {} for request {}",
                        self.action,
                        &self.controller.to_case(Case::Pascal),
                        self.uri
                    ));
                }
            }

            Ok(params)
        } else {
            Err(format!(
                "ERROR: action {} not found in controller {} for request {}",
                self.action,
                &self.controller.to_case(Case::Pascal),
                self.uri
            ))
        }
    }

    pub fn get_view(&self, app_data: &AppData) -> Result<String, String> {
        if let Some(actions) = app_data
            .views
            .get(self.controller.trim_end_matches("_controller"))
        {
            if let Some(view) = actions.get(&self.action) {
                return Ok(view.response.join(","));
            }
        }
        Err("not found".to_string())
    }
}

pub fn parse_routes(node: &Node) -> Result<Vec<Request>, String> {
    let mut buf = VecDeque::new();
    buf.push_back(node);
    let routes: Vec<Request> = Vec::new();
    while let Some(temp) = buf.pop_front() {}
    Err("failed to parse".to_owned())
}

#[cfg(test)]
mod routes_parsing {
    use lib_ruby_parser::Parser;

    use super::parse_routes;
    use super::Request;
    use super::RequestMethod;

    fn helper(input: &str) -> Box<lib_ruby_parser::Node> {
        Box::new(
            Parser::new(input.as_bytes(), Default::default())
                .do_parse()
                .ast
                .unwrap(),
        )
    }

    #[test]
    fn basic_parse() {
        let input = "
        Rails.application.routes.draw do
            get '/accounts/tags' => 'tags#index'
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            Request {
                method: RequestMethod::Post,
                prefix: "".to_string(),
                action: "index".to_string(),
                uri: "".to_owned(),
                controller: "tags".to_owned()
            }
        );
    }

    #[test]
    fn seperate_action_and_controller() {
        let input = "
        Rails.application.routes.draw do
            get 'profile', action: :show, controller: 'users'
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            Request {
                method: RequestMethod::Post,
                prefix: "".to_string(),
                action: "show".to_string(),
                uri: "".to_owned(),
                controller: "users".to_owned()
            }
        );
    }

    #[test]
    fn parse_hash_controller_action() {
        let input = "
        Rails.application.routes.draw do
            get 'profile', to: 'users#show'
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            Request {
                method: RequestMethod::Post,
                prefix: "".to_string(),
                action: "show".to_string(),
                uri: "".to_owned(),
                controller: "users".to_owned()
            }
        );
    }

    #[test]
    fn griddler() {
        let input = "
        Rails.application.routes.draw do
            mount_griddler
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn no_routes() {
        let input = "
        Rails.application.routes.draw do
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn no_routes_draw_found() {
        let input = "
        Rails.application.routes.draw do
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(
            result.is_err(),
            true,
            "threw some kind of error if no `Rails.application.routes.draw do` is found"
        );
    }

    #[test]
    fn basic_resources() {
        // https://guides.rubyonrails.org/routing.html#crud-verbs-and-actions
        // Rails routes are matched in the order they are specified,
        // so if you have a resources :photos above a get 'photos/poll' the show action's route for the resources line
        // will be matched before the get line.
        // To fix this, move the get line above the resources line so that it is matched first.

        // TODO: consider how rousource helpers will be considered as they expose stuff like:
        // resources :photos
        // photos_path -> /photos
        // new_photos_path -> /photos/new
        // etc.

        let input = "
        Rails.application.routes.draw do
            resources :photos
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn multiple_resources_in_a_given_line() {
        let input = "
        Rails.application.routes.draw do
            resources :photos, :books
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 12);
    }

    #[test]
    fn namespaces() {
        // https://guides.rubyonrails.org/routing.html#controller-namespaces-and-routing
        let input = "
        Rails.application.routes.draw do
            namespace :admin do
                resources :articles, :comments
            end
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 12);
    }

    #[test]
    fn scope_with_module() {
        // https://guides.rubyonrails.org/routing.html#controller-namespaces-and-routing
        let input = "
        Rails.application.routes.draw do
            scope module: 'admin' do
                resources :articles, :comments
            end
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 12);
    }

    #[test]
    fn resources_with_module() {
        // https://guides.rubyonrails.org/routing.html#controller-namespaces-and-routing
        let input = "
        Rails.application.routes.draw do
           resources :articles, module: 'admin'
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 6);
    }

    #[test]
    fn scope_without_module() {
        // https://guides.rubyonrails.org/routing.html#controller-namespaces-and-routing
        let input = "
        Rails.application.routes.draw do
            scope '/admin' do
                resources :articles, :comments
            end
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 12);
    }

    #[test]
    fn resources_with_path() {
        // https://guides.rubyonrails.org/routing.html#controller-namespaces-and-routing
        let input = "
        Rails.application.routes.draw do
            resources :articles, path: '/admin/articles'
        end
        ";
        let result = parse_routes(&helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");

        let result = result.unwrap();
        assert_eq!(result.len(), 12);
    }

    mod resources {
        // https://api.rubyonrails.org/v6.1.3.2/classes/ActionDispatch/Routing/Mapper/Resources.html#method-i-resources
        use super::*;

        #[test]
        fn resources_nested() {
            // https://guides.rubyonrails.org/routing.html#nested-resources
            let input = "
        Rails.application.routes.draw do
            resources :magazines do
                resources :ads
            end
        end
        ";
            let result = parse_routes(&helper(input));
            assert_eq!(result.is_ok(), true, "parsed okay");

            let result = result.unwrap();
            assert_eq!(result.len(), 6);
        }

        #[test]
        fn only_index_new_create() {
            // https://guides.rubyonrails.org/routing.html#nested-resources
            let input = "
        Rails.application.routes.draw do
            resources :comments, only: [:index, :new, :create]
        end
        ";
            let result = parse_routes(&helper(input));
            assert_eq!(result.is_ok(), true, "parsed okay");

            let result = result.unwrap();
            assert_eq!(result.len(), 3);
        }

        #[test]
        fn only_show_edit_update_destroy() {
            // https://guides.rubyonrails.org/routing.html#nested-resources
            let input = "
        Rails.application.routes.draw do
            resources :comments, only: [:show, :edit, :update, :destroy]
        end
        ";
            let result = parse_routes(&helper(input));
            assert_eq!(result.is_ok(), true, "parsed okay");

            let result = result.unwrap();
            assert_eq!(result.len(), 6);
        }

        #[test]
        fn shallow_on_child() {
            // https://guides.rubyonrails.org/routing.html#shallow-nesting
            let input = "
        Rails.application.routes.draw do
            resources :articles do
                resources :comments, shallow: true
            end
      
        end
        ";
            let result = parse_routes(&helper(input));
            assert_eq!(result.is_ok(), true, "parsed okay");

            let result = result.unwrap();
            assert_eq!(result.len(), 6);
        }

        #[test]
        fn shallow_on_parent() {
            // https://guides.rubyonrails.org/routing.html#shallow-nesting
            let input = "
        Rails.application.routes.draw do
            resources :comments, only: [:show, :edit, :update, :destroy]
        end
        ";
            let result = parse_routes(&helper(input));
            assert_eq!(result.is_ok(), true, "parsed okay");

            let result = result.unwrap();
            assert_eq!(result.len(), 6);
        }

        #[test]
        fn shallow_do() {
            // https://guides.rubyonrails.org/routing.html#shallow-nesting
            let input = "
        Rails.application.routes.draw do
            shallow do
                resources :articles do
                    resources :comments
                    resources :quotes
                    resources :drafts
                end
            end
        end
        ";
            let result = parse_routes(&helper(input));
            assert_eq!(result.is_ok(), true, "parsed okay");

            let result = result.unwrap();
            assert_eq!(result.len(), 6);
        }

        #[test]
        fn shallow_path() {
            // https://guides.rubyonrails.org/routing.html#shallow-nesting
            let input = "
        Rails.application.routes.draw do
            scope shallow_path: 'sekret' do
                resources :articles do
                    resources :comments, shallow: true
                end
            end
        end
        ";
            let result = parse_routes(&helper(input));
            assert_eq!(result.is_ok(), true, "parsed okay");

            let result = result.unwrap();
            assert_eq!(result.len(), 6);
        }

        #[test]
        fn shallow_prefix() {
            // https://guides.rubyonrails.org/routing.html#shallow-nesting
            let input = "
        Rails.application.routes.draw do
            scope shallow_prefix: 'sekret' do
                resources :articles do
                    resources :comments, shallow: true
                end
            end
        end
        ";
            let result = parse_routes(&helper(input));
            assert_eq!(result.is_ok(), true, "parsed okay");

            let result = result.unwrap();
            assert_eq!(result.len(), 6);
        }
    }
}
