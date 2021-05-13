use std::{
    clone,
    collections::{HashMap, VecDeque},
    fs,
    path::Path,
};

use lib_ruby_parser::{Node, Parser, ParserResult};
use walkdir::{DirEntry, WalkDir};

use crate::{
    types::{View, ViewType},
    utils,
};

fn parse_jbuiler_nodes(node: &Node, optional: bool, parent: &str) -> Vec<String> {
    let mut results = Vec::new();
    let mut buf = VecDeque::new();
    buf.push_back(node);
    while let Some(temp) = buf.pop_front() {
        match temp {
            Node::Begin(begin) => {
                for arg in &begin.statements {
                    buf.push_back(arg);
                }
            }
            Node::Block(begin) => {
                if let Node::Send(stat) = *begin.call.clone() {
                    if !stat.method_name.is_empty() && parent.ends_with(&stat.method_name) {
                        for arg in &begin.args {
                            buf.push_back(arg);
                        }
                        for arg in &begin.body {
                            buf.push_back(arg);
                        }
                    } else {
                        results.append(&mut parse_jbuiler_nodes(
                            temp,
                            optional,
                            &(parent.to_owned() + &stat.method_name.clone()),
                        ))
                    }
                } else {
                    buf.push_back(&*begin.call);
                    for arg in &begin.args {
                        buf.push_back(arg);
                    }
                    for arg in &begin.body {
                        buf.push_back(arg);
                    }
                }
            }
            Node::If(stat) => {
                stat.if_true.iter().for_each(|b| {
                    results.append(&mut parse_jbuiler_nodes(b, true, parent.clone()))
                });
                stat.if_false.iter().for_each(|a| {
                    results.append(&mut parse_jbuiler_nodes(a, false, parent.clone()))
                });
            }
            Node::Send(stat) => {
                let prefix = if optional { "?" } else { "" }.to_string();
                for arg in &stat.args {
                    if let Node::Ivar(_) = arg {
                    } else {
                        let temp = utils::parse_node_str(arg);
                        if temp == "unknown" {
                            buf.push_back(arg);
                        } else {
                            if parent.is_empty() {
                                results.push(format!("{}{}", prefix, &temp));
                            } else {
                                results.push(format!("{}.{}{}", parent, prefix, &temp));
                            }
                        }
                    }
                }
                if let Some(recv) = stat.recv.clone() {
                    if let Node::Send(is_json) = *recv.clone() {
                        if is_json.method_name == "json" {
                            if stat.method_name != "call" {
                                if parent.is_empty() {
                                    results.push(format!("{}{}", prefix, stat.method_name));
                                } else {
                                    results
                                        .push(format!("{}.{}{}", parent, prefix, stat.method_name));
                                }
                            }
                        }
                    } else {
                    }
                    // else if let Node::Begin(_) = *recv.clone() {
                    //     // results.append(&mut parse_jbuiler_nodes(
                    //     //     &*(stat.recv.clone().unwrap()).clone(),
                    //     //     optional,
                    //     //     &(parent.to_owned() + &stat.method_name.clone()),
                    //     // ))

                    //     if !stat.method_name.is_empty() && parent.ends_with(&stat.method_name) {
                    //         buf.push_back(value)
                    //     } else {
                    //         results.append(&mut parse_jbuiler_nodes(
                    //             temp,
                    //             optional,
                    //             &(parent.to_owned() + &stat.method_name.clone()),
                    //         ))
                    //     }
                    // }
                }
            }
            Node::Procarg0(_) => {}
            Node::Args(args) => args.args.iter().for_each(|f| buf.push_back(f)),
            Node::Ivar(_) => {}
            // conditional send e.g. foo&.id
            Node::CSend(_) => {
                println!("")
            }
            _ => {
                // panic!("{:?}", temp);
            }
        }
    }

    results
}

fn parse_jbuilder(
    parser: ParserResult,
    action: String,
    controller: String,
) -> Result<View, String> {
    if let Some(ast) = parser.ast {
        Ok(View {
            controller,
            method: action,
            response: parse_jbuiler_nodes(&ast, false, ""),
            view_type: ViewType::Jbuilder,
        })
    } else {
        Err("empty view".to_owned())
    }
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub fn parse_view_files(
    path: &Path,
    views: &mut HashMap<String, HashMap<String, View>>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in WalkDir::new(path)
        .into_iter()
        .filter_entry(|e| -> bool { !is_hidden(e) })
        .filter_map(|e| e.ok())
    {
        let f = entry.path();
        let name = f.display().to_string();
        if f.is_file() && (name.ends_with(".jbuilder") || name.ends_with(".jb")) {
            let controller = f
                .parent()
                .unwrap()
                .display()
                .to_string()
                .split("/")
                .last()
                .unwrap()
                .to_string();
            let action = f
                .display()
                .to_string()
                .split("/")
                .last()
                .unwrap()
                .to_string();
            let views_controller = views.entry(controller.clone()).or_insert(HashMap::new());
            let parser = Parser::new(&fs::read(entry.path())?, Default::default()).do_parse();

            if action.ends_with(".jbuilder") {
                views_controller.insert(
                    action.trim_end_matches(".jbuilder").to_owned(),
                    parse_jbuilder(parser, action, controller)?,
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod views_tests {
    use lib_ruby_parser::Parser;

    use pretty_assertions::assert_eq;

    use super::parse_jbuiler_nodes;

    fn helper(input: &str) -> Vec<String> {
        let mut results = parse_jbuiler_nodes(
            &Parser::new(input.as_bytes(), Default::default())
                .do_parse()
                .ast
                .unwrap(),
            false,
            "",
        );
        results.sort();
        results
    }
    #[test]
    fn blocks() {
        let input = "
        json.uploads  @data.uploads do | upload |
            json.(upload, :id, :stored_filename, :user_filename, :file_type)
        end
        ";

        assert_eq!(
            helper(input),
            [
                "uploads.file_type".to_owned(),
                "uploads.id".to_owned(),
                "uploads.stored_filename".to_owned(),
                "uploads.upload".to_owned(),
                "uploads.user_filename".to_owned(),
            ]
        );
    }

    #[test]
    fn named_block_with_if() {
        let input = "
        json.uploads  @data.uploads do | upload |
            json.(upload, :id, :stored_filename, :user_filename, :file_type)
            if @options && @options[:include_upload_links]
                json.url upload.download_link
            end
        end
        ";

        assert_eq!(
            helper(input),
            [
                "uploads.?url".to_owned(),
                "uploads.file_type".to_owned(),
                "uploads.id".to_owned(),
                "uploads.stored_filename".to_owned(),
                "uploads.upload".to_owned(),
                "uploads.user_filename".to_owned(),
            ]
        );
    }

    #[test]
    fn named_begin() {
        let input = "
        json.editor do
            json.name @data.editor&.id
            json.id @data.editor&.id
        end
        ";

        assert_eq!(
            helper(input),
            ["editor.id".to_owned(), "editor.name".to_owned(),]
        );
    }

    #[test]
    fn basic() {
        let input = "
        json.(@data, :id, :title, :description)
        ";

        assert_eq!(
            helper(input),
            [
                "description".to_owned(),
                "id".to_owned(),
                "title".to_owned(),
            ]
        );
    }

    #[test]
    fn if_statement() {
        let input = "
        if @data.owner
            json.(@data, :read_count)
        end
        ";

        assert_eq!(helper(input), ["?read_count".to_owned()]);
    }

    #[test]
    fn conditional_based_send() {
        let input = "
        json.admin permission.admin?
        ";

        assert_eq!(helper(input), ["admin".to_owned()]);
    }
}
