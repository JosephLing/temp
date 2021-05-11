use std::{collections::VecDeque, str::FromStr};

use lib_ruby_parser::Node;

#[derive(Debug, PartialEq)]
pub enum RequestMethod {
    GET,
    POST,
    DELETE,
    PUT,
    PATCH,
    OPTIONS,
}

impl FromStr for RequestMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "GET" => RequestMethod::GET,
            "POST" => RequestMethod::POST,
            "DELETE" => RequestMethod::DELETE,
            "PUT" => RequestMethod::PUT,
            "PATCH" => RequestMethod::PATCH,
            "OPTIONS" => RequestMethod::OPTIONS,
            _ => Err(format!("unknown Request method '{}'", s.clone()))?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Request {
    method: RequestMethod,
    prefix: String,
    uri: String,
    pub controller: String,
    pub action: String,
}

impl std::fmt::Display for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {}", self.method, self.uri)
    }
}

pub fn parse_routes(node: Node) -> Result<Vec<Request>, String> {
    let mut buf = VecDeque::new();
    buf.push_back(node);
    let routes: Vec<Request> = Vec::new();
    while let Some(temp) = buf.pop_front() {
        
    }
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

            mount_griddler

            get '/accounts/tags' => 'tags#get_all_custom_account_tags'

        end
        ";
        let result = parse_routes(*helper(input));
        assert_eq!(result.is_ok(), true, "parsed okay");
        
        let result = result.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(
            result[0],
            Request {
                method: RequestMethod::POST,
                prefix: "email_processor".to_string(),
                uri: "/email_processor".to_string(),
                controller: "griddler/emails".to_string(),
                action: "create".to_string(),
            }
        );
    }
}
