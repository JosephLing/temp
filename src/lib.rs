mod params;
mod routes;
mod types;
mod utils;
mod views;

use types::{ActionKinds, AppData, Concern, Controller, HelperModule, MethodDetails};

use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::Path,
};

use lib_ruby_parser::{
    nodes::{Class, Send},
    Node, Parser,
};

use utils::{get_node_name, parse_name, parse_superclass};
use walkdir::{DirEntry, WalkDir};

use crate::routes::{parse_routes, Request};

#[derive(Debug)]
enum File {
    Controller(Controller),
    Module(HelperModule),
    Concern(Concern),
    None,
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with('.'))
        .unwrap_or(false)
}

fn get_method_details_from_optional(
    optional_args: Option<Box<Node>>,
    name: String,
    methods: &mut Vec<MethodDetails>,
) -> Option<MethodDetails> {
    if let Some(arg) = optional_args {
        methods.push(params::create_method_details(arg, name, Vec::new()));
    }

    None
}

fn parse_actions(send_thing: Send, actions: &mut Vec<(ActionKinds, String)>) {
    match send_thing.method_name.as_str() {
        "before_action" => {
            for arg in &send_thing.args {
                actions.push((ActionKinds::BeforeAction, utils::parse_node_str(arg)));
            }
        }
        "around_action" => {
            for arg in &send_thing.args {
                actions.push((ActionKinds::AroundAction, utils::parse_node_str(arg)));
            }
        }
        "rescue_from" => {
            for arg in &send_thing.args {
                actions.push((ActionKinds::RescueFrom, utils::parse_node_str(arg)));
            }
        }
        _ => {
            // for arg in &send_thing.args {
            //     actions.push((
            //         ActionKinds::Custom(send_thing.method_name.clone()),
            //         utils::parse_node_str(arg),
            //     ));
            // }
        }
    }
}

fn parse_class(class: Class, module: String) -> Result<File, String> {
    let name = parse_name(*class.name);
    let superclass = parse_superclass(class.superclass);
    if superclass.is_empty() {
        Err("single file classes not supported".to_string())
    } else if superclass != "StandardError" {
        if let Some(body) = class.body {
            let mut methods = Vec::new();
            let mut includes = Vec::new();
            let mut actions = Vec::new();
            match *body {
                // def and defs .name and we need to consider the argument names it takes.... but I haven't thought about args
                Node::Def(stat) => {
                    get_method_details_from_optional(stat.body, stat.name, &mut methods);
                }
                Node::Defs(stat) => {
                    get_method_details_from_optional(stat.body, stat.name, &mut methods);
                }

                Node::Begin(stat) => {
                    for stat in stat.statements {
                        match stat {
                            Node::Send(send_thing) => match send_thing.method_name.as_str() {
                                "require" => {}
                                "include" => {
                                    for arg in &send_thing.args {
                                        let temp = utils::parse_node_str(arg);
                                        if !temp.starts_with("ActionController") {
                                            includes.push(temp);
                                        }
                                    }
                                }
                                "private" => {}
                                "protected" => {}
                                _ => parse_actions(send_thing, &mut actions),
                            },
                            Node::Def(stat) => {
                                get_method_details_from_optional(
                                    stat.body,
                                    stat.name,
                                    &mut methods,
                                );
                            }
                            Node::Defs(stat) => {
                                get_method_details_from_optional(
                                    stat.body,
                                    stat.name,
                                    &mut methods,
                                );
                            }
                            Node::Casgn(_) => {
                                // END_USER_ALLOWED_SETTINGS
                            }
                            _ => {
                                panic!("ahhhh {:?}", stat);
                            }
                        }
                    }
                }
                _ => {
                    panic!("oh no {:?}", body);
                }
            }
            Ok(File::Controller(Controller {
                name,
                parent: superclass,
                methods,
                actions,
                include: includes,
                module: if module.is_empty() {
                    None
                } else {
                    Some(module)
                },
            }))
        } else {
            Ok(File::Controller(Controller {
                name,
                parent: superclass,
                methods: Vec::new(),
                actions: Vec::new(),
                include: Vec::new(),
                module: if module.is_empty() {
                    None
                } else {
                    Some(module)
                },
            }))
        }
    } else {
        Ok(File::None)
    }
}

fn parse_file(node: Node) -> Result<Vec<File>, String> {
    let mut files = Vec::new();
    let mut buf = VecDeque::new();
    let mut module_name = "".to_owned();
    let mut module_names = VecDeque::new();
    buf.push_back(node);
    while let Some(temp) = buf.pop_front() {
        if let Some(new_name) = module_names.pop_front() {
            module_name = new_name;
        }
        match temp {
            Node::Module(module) => {
                if let Some(body) = module.body {
                    buf.push_back(*body);
                }
                module_names.push_back(module_name.clone() + &parse_name(*module.name));
            }
            Node::Def(stat) => {
                let mut methods = Vec::new();
                get_method_details_from_optional(stat.body, stat.name, &mut methods);
                files.push(File::Module(HelperModule {
                    name: module_name.clone(),
                    methods,
                }))
            }
            Node::Defs(stat) => {
                let mut methods = Vec::new();
                get_method_details_from_optional(stat.body, stat.name, &mut methods);
                files.push(File::Module(HelperModule {
                    name: module_name.clone(),
                    methods,
                }))
            }
            Node::Class(class) => files.push(parse_class(class, module_name.clone())?),
            Node::Begin(begin) => {
                let mut helper_found = false;
                let mut concern_found = false;
                let mut methods = Vec::<MethodDetails>::new();
                let mut actions = Vec::new();
                for stat in begin.statements {
                    match stat {
                        Node::Module(module) => {
                            if let Some(body) = module.body {
                                buf.push_back(*body);
                            }
                            module_names.push_back(module_name.clone() + &parse_name(*module.name));
                        }
                        Node::Class(class) => {
                            files.push(parse_class(class, module_name.clone())?);
                        }
                        Node::Send(send) => {
                            match send.method_name.as_str() {
                                "extend" => {
                                    if send.args.len() == 1
                                        && get_node_name(&send.args[0])? == "ActiveSupport::Concern"
                                    {
                                        concern_found = true;
                                        break;
                                    } else {
                                        return Err("unsupported 'extend' found".to_owned());
                                    }
                                }

                                //TODO: require
                                "require" => {}
                                "private" => {}
                                "private_class_method" => {
                                    // TODOD: check if the method is an arg for this!!!
                                }

                                _ => {
                                    return Err(format!("unexpected 'send' in file {:?}", send));
                                }
                            }
                        }
                        Node::Casgn(_) => {}
                        Node::Def(stat) => {
                            get_method_details_from_optional(stat.body, stat.name, &mut methods);
                            if !concern_found {
                                helper_found = true;
                            }
                        }
                        Node::Defs(stat) => {
                            get_method_details_from_optional(stat.body, stat.name, &mut methods);
                            if !concern_found {
                                helper_found = true;
                            }
                        }
                        Node::Block(block) => {
                            if concern_found {
                                if block.args.is_some() {
                                    return Err("included only supported in concerns".to_owned());
                                }
                                if let Node::Send(stat) = *block.call {
                                    if stat.method_name == "included" {
                                        if let Some(arg) = block.args {
                                            if let Node::Send(action_stat) = *arg {
                                                parse_actions(action_stat, &mut actions)
                                            }
                                        }
                                    } else {
                                        return Err(format!(
                                            "expected 'included' call got instead '{}'",
                                            stat.method_name
                                        ));
                                    }
                                } else {
                                    return Err("expected send for call block".to_owned());
                                }
                            } else {
                                return Err(
                                    "blocks are only supported inside concerns atm".to_owned()
                                );
                            }
                        }
                        _ => {
                            println!("{:?}", stat);
                            return Err(
                                "expected file to have a class or module inside it".to_string()
                            );
                        }
                    }
                }
                if helper_found {
                    files.push(File::Module(HelperModule {
                        name: module_name.clone(),
                        methods,
                    }));
                } else if concern_found {
                    files.push(File::Concern(Concern {
                        name: module_name.clone(),
                        methods,
                        actions,
                    }));
                }
            }
            _ => {
                println!("{:?}", temp);
                return Err("error unknown syntax found in file HERE ".to_string());
            }
        }
    }

    Ok(files)
}

fn parse_files(
    path: &Path,
    controllers: &mut HashMap<String, Controller>,
    concerns: &mut HashMap<String, Concern>,
    helpers: &mut HashMap<String, HelperModule>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = Vec::new();
    let mut file_count = 0;
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| -> bool { !is_hidden(e) })
        .filter_map(|e| e.ok())
    {
        let f = entry.path();
        let name = f.display().to_string();
        if f.is_file() && name.ends_with(".rb") {
            let parser = Parser::new(&fs::read(entry.path())?, Default::default()).do_parse();
            match parser.ast {
                Some(node) => {
                    let result = parse_file(node);
                    match result {
                        Ok(result) => {
                            for cat in result {
                                match cat {
                                    File::Controller(controller) => {
                                        controllers.insert(controller.name.clone(), controller);
                                    }
                                    File::Module(module) => {
                                        helpers.insert(module.name.clone(), module);
                                    }
                                    File::Concern(concern) => {
                                        concerns.insert(concern.name.clone(), concern);
                                    }
                                    File::None => {
                                        errors.push((
                                            name.clone(),
                                            format!("invalid parsed file result found {:?}", cat),
                                        ));
                                    }
                                }
                            }
                        }
                        Err(err) => errors.push((name, err)),
                    }
                }
                None => errors.push((
                    name,
                    format!(
                        "Empty file found, found {} syntax errors",
                        parser.diagnostics.len()
                    ),
                )),
            }
        }
        file_count += 1;
    }

    if !errors.is_empty() {
        println!(
            "Got {} errors out of a total of {} files",
            errors.len(),
            file_count
        );

        for (file_error, error) in errors {
            println!("{} {:?}", file_error, error);
        }
    }

    Ok(())
}

pub fn compute(root: &Path) -> Result<AppData, Box<dyn std::error::Error>> {
    let mut route_path = root.to_path_buf();
    route_path.push("test.routes");
    if !route_path.exists() {
        return Err("no test.routes file found in root of rails project directory, run `bundle exec rails r routes > test.routes` to generate the file".into());
    }

    let mut routes: HashMap<String, Request> = HashMap::new();
    for route in parse_routes(&fs::read_to_string(route_path).unwrap())? {
        routes.insert(route.uri.clone(), route);
    }

    let mut app_data = AppData {
        concerns: HashMap::new(),
        helpers: HashMap::new(),
        controllers: HashMap::new(),
        routes,
        views: HashMap::new(),
    };

    let mut app_dir = root.to_path_buf();
    app_dir.push("app");

    let mut helpers_path = app_dir.clone();
    helpers_path.push("helpers");

    let mut controllers_path = app_dir.clone();
    controllers_path.push("controllers");

    let mut view_path = app_dir;
    view_path.push("views");

    parse_files(
        &controllers_path,
        &mut app_data.controllers,
        &mut app_data.concerns,
        &mut app_data.helpers,
    )?;
    parse_files(
        &helpers_path,
        &mut app_data.controllers,
        &mut app_data.concerns,
        &mut app_data.helpers,
    )?;

    views::parse_view_files(&view_path, &mut app_data.views)?;

    Ok(app_data)
}

#[cfg(test)]
mod parse_class_tests {
    use lib_ruby_parser::{Node, Parser};

    use crate::parse_class;

    fn helper(input: &str) -> Box<lib_ruby_parser::Node> {
        Box::new(
            Parser::new(input.as_bytes(), Default::default())
                .do_parse()
                .ast
                .unwrap(),
        )
    }
    #[test]
    fn basic() {
        let input = "
        class ApplicationController < ActionController::API 
            include HttpResponses

            before_action :auth_check

            def auth_check
                return unless params[:auth_token] == 1
            end
        end
        ";
        if let Node::Class(value) = *helper(input) {
            println!("{:?}", parse_class(value, "".to_string()));

            // fail to just
            assert_eq!(false, true);
        }
    }
}
