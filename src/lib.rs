use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use lib_ruby_parser::{nodes::Class, Node, Parser};

use params::MethodDetails;
use utils::{get_node_name, parse_name, parse_superclass};
use walkdir::{DirEntry, WalkDir};

use crate::routes::parse_routes;

mod params;
mod routes;
mod utils;
#[derive(Debug)]
struct Controller {
    pub name: String,
    pub parent: String,
    pub methods: Vec<MethodDetails>,
    pub actions: Vec<String>,
    pub include: Vec<String>,
    pub module: Option<String>,
    // ignoring requires for now
}

#[derive(Debug)]
struct HelperModule {
    pub name: String,
    pub methods: Vec<MethodDetails>,
}
#[derive(Debug)]
struct Concern {
    pub name: String,
    pub methods: Vec<MethodDetails>,
    pub actions: Vec<String>, // TODO: work out what this looks like
}

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
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn parse_class(class: Class, module: String) -> Result<File, String> {
    let name = parse_name(class.name);
    let superclass = parse_superclass(class.superclass);
    if superclass.is_empty() {
        Err("single file classes not supported".to_string())
    } else if superclass != "StandardError" {
        if let Some(body) = class.body {
            // methods
            params::search_for_param(body);
        }
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
                module_names.push_back(module_name.clone() + &parse_name(module.name));
            },
            Node::Def(_def) => {
                files.push(File::Module(HelperModule {
                    name: module_name.clone(),
                    methods: Vec::new(),
                }))
            },
            Node::Defs(_def) => {
                files.push(File::Module(HelperModule {
                    name: module_name.clone(),
                    methods: Vec::new(),
                }))
            }
            Node::Class(class) => files.push(parse_class(class, module_name.clone())?),
            Node::Begin(begin) => {
                let mut helper_found = false;
                for stat in begin.statements {
                    match stat {
                        Node::Module(module) => {
                            if let Some(body) = module.body {
                                buf.push_back(*body);
                            }
                            module_names.push_back(module_name.clone() + &parse_name(module.name));
                        }
                        Node::Class(class) => {
                            files.push(parse_class(class, module_name.clone())?);
                        }
                        Node::Send(send) => {
                            if send.method_name == "require" {
                                // TODO: no validation but keeping track of how dependencies are used, would be useful
                            } 
                            // ignoring visibility for now
                            else if send.method_name == "private" {
                            } else if send.method_name == "private_class_method" {
                                // do nothing
                            } 
                            
                            else if send.method_name == "extend" {
                                let mut found = false;
                                for arg in &send.args {
                                    if get_node_name(&arg)? == "ActiveSupport::Concern" {
                                        files.push(File::Concern(Concern {
                                            name: module_name.clone(),
                                            methods: Vec::new(),
                                            actions: Vec::new(),
                                        }));
                                        found = true;
                                        break;
                                    } else {
                                        return Err("unsupported 'extend' found".to_owned());
                                    }
                                }
                                if found {
                                    break;
                                }
                            } else {
                                return Err(format!("unexpected 'send' in file {:?}", send));
                            }
                        }
                        Node::Casgn(_) => {}
                        Node::Def(_) => {
                            helper_found = true;
                        }
                        Node::Defs(_) => {
                            helper_found = true;
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
                        methods: Vec::new(),
                    }))
                }
            }
            _ => {
                println!("{:?}", temp);
                return Err("error unknown syntax found in file HERE ".to_string())
            },
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

pub fn compute(root: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    // TODO: abstract these out so unit tests can written... ah more work but will help
    let mut concerns: HashMap<String, Concern> = HashMap::new();
    let mut helper: HashMap<String, HelperModule> = HashMap::new();
    let mut controller: HashMap<String, Controller> = HashMap::new();

    let mut route_path = root.clone();
    route_path.push("config");
    route_path.push("routes.rb");
    let routes = parse_routes(
        Parser::new(&fs::read(route_path)?, Default::default())
            .do_parse()
            .ast
            .unwrap(),
    );
    println!("Routes {:?}", routes);

    let mut app_dir = root.clone();
    app_dir.push("app");

    let mut helper_path = app_dir.clone();
    helper_path.push("helpers");

    let mut controllers_path = app_dir.clone();

    controllers_path.push("controllers");

    parse_files(
        &controllers_path,
        &mut controller,
        &mut concerns,
        &mut helper,
    )?;
    parse_files(&helper_path, &mut controller, &mut concerns, &mut helper)?;
    println!("--- Controllers ---");

    for (_name, con) in controller {
        println!("{:?} {} {}", con.module, con.name, con.parent)
    }
    println!("--- Helpers ---");

    for (name, _con) in helper {
        println!("{}", name)
    }

    println!("--- Concerns ---");
    for (name, _con) in concerns {
        println!("{}", name)
    }

    Ok(())
}
