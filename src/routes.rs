use crate::types::AppData;
use convert_case::{Case, Casing};
use std::collections::HashSet;
use std::str::FromStr;

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

pub fn parse_routes(input: &str) -> Result<Vec<Request>, String> {
    if input.is_empty() {
        Err("input is empty".to_string())
    } else {
        let mut routes = Vec::new();
        let lines: Vec<Vec<String>> = input
            .lines()
            .skip(1)
            .into_iter()
            .map(|f| {
                f.split_whitespace()
                    .map(|e| e.to_string())
                    .filter(|e| !e.is_empty())
                    .collect()
            })
            .collect();

        // this ugly mess is grabbing the valid feilds but ignoring the last one if an extra resource thing is added on to the end as I don't know what it does
        for line in &lines {
            if line.len() == 5 {
            } else if line.len() == 4 {
                if let Ok(temp2) = RequestMethod::from_str(&line[0]) {
                    let temp = line[2].split('#').collect::<Vec<&str>>();
                    if temp.len() != 2 {
                        return Err(format!(
                            "could not find action on the contorller {}",
                            line[2]
                        ));
                    }

                    routes.push(Request {
                        method: temp2,
                        prefix: "".to_string(),
                        uri: line[1].replace("(.:format)", ""),
                        controller: temp[0].to_string() + "_controller",
                        action: temp[1].to_string(),
                    })
                } else {
                    let temp = line[3].split('#').collect::<Vec<&str>>();
                    if temp.len() != 2 {
                        return Err(format!(
                            "could not find action on the contorller {}",
                            line[3]
                        ));
                    }

                    routes.push(Request {
                        method: RequestMethod::from_str(&line[1])?,
                        prefix: line[0].clone(),
                        uri: line[2].replace("(.:format)", ""),
                        controller: temp[0].to_string() + "_controller",
                        action: temp[1].to_string(),
                    })
                }
            } else if line.len() == 3 {
                let temp = line[2].split('#').collect::<Vec<&str>>();
                if temp.len() != 2 {
                    return Err(format!(
                        "could not find action on the contorller {}",
                        line[2]
                    ));
                }

                routes.push(Request {
                    method: RequestMethod::from_str(&line[0])?,
                    prefix: "".to_string(),
                    uri: line[1].replace("(.:format)", ""),
                    controller: temp[0].to_string() + "_controller",
                    action: temp[1].to_string(),
                })
            } else {
                println!("panic {:?}", line);
            }
        }

        Ok(routes)
    }
}

#[cfg(test)]
mod routes_parsing {
    use super::parse_routes;
    use super::Request;
    use super::RequestMethod;

    #[test]
    fn parse() {
        let input = "Prefix Verb    URI Pattern                                                                              Controller#Action
        email_processor POST    /email_processor(.:format)                                                               griddler/emails#create
            dog_form GET     /dog/form(.:format)                                                                   dog_forms#show
                        PATCH   /dog/form(.:format)                                                                   dog_forms#update
                        PUT     /dog/form(.:format)                                                                   dog_forms#update
                        POST    /dog/form(.:format)                                                                   dog_forms#create
          dog_styles GET     /dogs/:dog_id/styles(.:format)                                                     dogs/styles#index
                        POST    /dogs/:dog_id/styles(.:format)                                                     dogs/styles#create
       new_dog_style GET     /dogs/:dog_id/styles/new(.:format)                                                 dogs/styles#new
        ";

        assert_eq!(parse_routes(input).is_ok(), true, "successfully parse");
        assert_eq!(parse_routes(input).unwrap().len(), 8);
        assert_eq!(
            parse_routes(input).unwrap()[0],
            Request {
                method: RequestMethod::Post,
                prefix: "email_processor".to_string(),
                uri: "/email_processor".to_string(),
                controller: "griddler/emails_controller".to_string(),
                action: "create".to_string(),
            }
        );

        assert_eq!(
            parse_routes(input).unwrap()[2],
            Request {
                method: RequestMethod::Patch,
                prefix: "".to_string(),
                uri: "/dog/form".to_string(),
                controller: "dog_forms_controller".to_string(),
                action: "update".to_string(),
            }
        );
    }
}
