use crate::utils::{self, parse_node_str, parse_optional_name};
use std::collections::{HashMap, HashSet, VecDeque};

use lib_ruby_parser::{
    nodes::{self, Index},
    Node,
};
#[derive(Debug, PartialEq)]
enum SendTypes {
    ParamsPermit,
    ParamsRequire,
    ParamsRequirePermit,
    Invalid,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodDetails {
    pub name: String,
    pub args: Vec<String>,
    pub params: HashSet<String>,
    pub headers: Vec<(String, String)>, // TODO: need to implement this one

    pub instance_varaibles: HashSet<String>, // implemented
    pub local_varaibles: HashMap<String, usize>, // implemented

    // method name and method indexes
    pub method_calls: Vec<(String, Vec<String>)>, // is nearly done
    pub renders: Vec<(String, String)>,           // TODO: implement this one
}

fn handle_vector_of_nodes(statements: Vec<Node>, buf: &mut VecDeque<Box<Node>>) {
    for stat in &statements {
        buf.push_back(Box::new(stat.clone()));
    }
}

fn handle_optional_node(body: &Option<Box<Node>>, buf: &mut VecDeque<Box<Node>>) {
    if let Some(body) = body {
        buf.push_back((*body).clone());
    }
}

// doesn't support inline methods and singleton classes
// search for: param and headers (ignores payload)
pub fn create_method_details(
    statement: Box<Node>,
    method_name: String,
    args: Vec<String>,
) -> MethodDetails {
    let mut params = HashSet::new();
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut instance_varaibles: HashSet<String> = HashSet::new();
    let mut method_calls: Vec<(String, Vec<String>)> = Vec::new();
    let mut local_varaibles: HashMap<String, usize> = HashMap::new();
    let renders: Vec<(String, String)> = Vec::new();

    let mut buf = VecDeque::new();

    buf.push_back(statement);

    while let Some(temp) = buf.pop_front() {
        match *temp {
            Node::Alias(stat) => buf.push_back(stat.from),

            Node::And(stat) => {
                buf.push_back(stat.lhs);
                buf.push_back(stat.rhs);
            }
            Node::AndAsgn(stat) => buf.push_back(stat.value),

            Node::Array(stat) => handle_vector_of_nodes(stat.elements, &mut buf),
            Node::ArrayPattern(stat) => handle_vector_of_nodes(stat.elements, &mut buf),
            Node::ArrayPatternWithTail(stat) => handle_vector_of_nodes(stat.elements, &mut buf),

            Node::Begin(stat) => handle_vector_of_nodes(stat.statements, &mut buf),

            // note: ignore optional elements of block here
            Node::Block(stat) => handle_optional_node(&stat.body, &mut buf),
            Node::BlockPass(stat) => buf.push_back(stat.value),

            // Node::Case(stat) => {}
            // Node::CaseMatch(stat) => {}
            // Node::Casgn(stat) => {}
            // Node::Cbase(stat) => {}
            Node::Const(stat) => handle_optional_node(&stat.scope, &mut buf),

            Node::ConstPattern(stat) => buf.push_back(stat.pattern),

            Node::CSend(stat) => handle_vector_of_nodes(stat.args, &mut buf),

            // We don't currently use @@var style so not handling it
            // Node::Cvar(stat) => {}
            // Node::Cvasgn(stat) => {}
            Node::Defined(stat) => buf.push_back(stat.value),

            Node::Dstr(stat) => handle_vector_of_nodes(stat.parts, &mut buf),
            Node::Dsym(stat) => handle_vector_of_nodes(stat.parts, &mut buf),

            Node::EFlipFlop(stat) => {
                handle_optional_node(&stat.left, &mut buf);
                handle_optional_node(&stat.right, &mut buf)
            }

            Node::Ensure(stat) => {
                handle_optional_node(&stat.ensure, &mut buf);
                handle_optional_node(&stat.body, &mut buf)
            }

            Node::Erange(stat) => {
                handle_optional_node(&stat.left, &mut buf);
                handle_optional_node(&stat.right, &mut buf)
            }

            Node::FindPattern(stat) => handle_vector_of_nodes(stat.elements, &mut buf),

            Node::For(stat) => {
                buf.push_back(stat.iterator);
                buf.push_back(stat.iteratee);
                handle_optional_node(&stat.body, &mut buf);
            }

            // global vars $var = 1
            // Node::Gvar(stat) => {}
            // Node::Gvasgn(stat) => {}
            Node::Hash(stat) => handle_vector_of_nodes(stat.pairs, &mut buf),
            Node::HashPattern(stat) => handle_vector_of_nodes(stat.elements, &mut buf),

            Node::If(stat) => {
                buf.push_back(stat.cond);
                handle_optional_node(&stat.if_true, &mut buf);
                handle_optional_node(&stat.if_false, &mut buf)
            }
            Node::IfGuard(stat) => buf.push_back(stat.cond),
            Node::IFlipFlop(stat) => {
                handle_optional_node(&stat.left, &mut buf);
                handle_optional_node(&stat.right, &mut buf);
            }
            Node::IfMod(stat) => {
                buf.push_back(stat.cond);
                handle_optional_node(&stat.if_true, &mut buf);
                handle_optional_node(&stat.if_false, &mut buf)
            }
            Node::IfTernary(stat) => {
                buf.push_back(stat.cond);
                buf.push_back(stat.if_true);
                buf.push_back(stat.if_false);
            }

            Node::Index(stat) => {
                // recv is params
                // index
                if let Some(data) = params_index(stat) {
                    for item in data {
                        params.insert(item);
                    }
                }
            }

            Node::IndexAsgn(stat) => {
                if let Node::Send(send) = *stat.recv {
                    if send.method_name == "headers" {
                        println!("args {:?}", send.args);
                        for index in stat.indexes {
                            match index {
                                Node::Str(value) => {
                                    headers.push((
                                        value.value.to_string_lossy(),
                                        parse_optional_name(&stat.value),
                                    ));
                                }
                                _ => {}
                            }
                        }
                    } else {
                        handle_vector_of_nodes(send.args, &mut buf);
                        handle_optional_node(&send.recv, &mut buf);
                    }
                } else {
                    buf.push_back(stat.recv);
                    handle_vector_of_nodes(stat.indexes, &mut buf);
                }
            }

            Node::InPattern(stat) => {
                buf.push_back(stat.pattern);
                handle_optional_node(&stat.guard, &mut buf);
                handle_optional_node(&stat.body, &mut buf)
            }

            Node::Irange(stat) => {
                handle_optional_node(&stat.left, &mut buf);
                handle_optional_node(&stat.right, &mut buf);
            }

            // TODO: do we want to keep track of instance varaibles?? seems like it is isn't necessary
            // Node::Ivar(stat) => stat.name,
            Node::Ivasgn(stat) => {
                instance_varaibles.insert(stat.name);

                handle_optional_node(&stat.value, &mut buf)
            }

            Node::Kwargs(stat) => handle_vector_of_nodes(stat.pairs, &mut buf),
            Node::KwBegin(stat) => handle_vector_of_nodes(stat.statements, &mut buf),
            Node::Kwoptarg(stat) => buf.push_back(stat.default),
            Node::Kwsplat(stat) => buf.push_back(stat.value),

            Node::Lvar(stat) => {
                // parser already handles local varaible assignment and access
                // therefore we can keep track of it
                // verbose due to: avoiding mutable_borrow_reservation_conflict see: https://github.com/rust-lang/rust/issues/59159#
                let mut var_exists = false;
                let v = if let Some(value) = local_varaibles.get(&stat.name) {
                    var_exists = true;
                    *value
                } else {
                    0
                };

                if var_exists {
                    local_varaibles.insert(stat.name, v + 1);
                }
            }

            // specail case for headers and payload!!!!
            Node::Lvasgn(stat) => {
                let v = if let Some(value) = local_varaibles.get(&stat.name) {
                    *value
                } else {
                    0
                };
                local_varaibles.insert(stat.name, v);
                handle_optional_node(&stat.value, &mut buf)
            }

            Node::Masgn(stat) => {
                buf.push_back(stat.lhs);
                buf.push_back(stat.rhs);
            }

            Node::MatchAlt(stat) => {
                buf.push_back(stat.lhs);
                buf.push_back(stat.rhs);
            }
            Node::MatchAs(stat) => buf.push_back(stat.value),
            Node::MatchPattern(stat) => {
                buf.push_back(stat.value);
                buf.push_back(stat.pattern)
            }
            Node::MatchPatternP(stat) => {
                buf.push_back(stat.value);
                buf.push_back(stat.pattern)
            }
            Node::MatchRest(stat) => handle_optional_node(&stat.name, &mut buf),
            Node::MatchWithLvasgn(stat) => {
                buf.push_back(stat.re);
                buf.push_back(stat.value)
            }

            Node::Mlhs(stat) => handle_vector_of_nodes(stat.items, &mut buf),

            Node::Next(stat) => handle_vector_of_nodes(stat.args, &mut buf),

            Node::Numblock(stat) => buf.push_back(stat.body),

            Node::OpAsgn(stat) => {
                buf.push_back(stat.recv);
                buf.push_back(stat.value)
            }
            Node::Optarg(stat) => buf.push_back(stat.default),

            Node::Or(stat) => {
                buf.push_back(stat.lhs);
                buf.push_back(stat.rhs);
            }
            Node::OrAsgn(stat) => {
                buf.push_back(stat.recv);
                buf.push_back(stat.value)
            }

            Node::Pair(stat) => {
                buf.push_back(stat.key);
                buf.push_back(stat.value)
            }

            Node::Pin(stat) => buf.push_back(stat.var),

            Node::Postexe(stat) => handle_optional_node(&stat.body, &mut buf),
            Node::Preexe(stat) => handle_optional_node(&stat.body, &mut buf),
            Node::Procarg0(stat) => handle_vector_of_nodes(stat.args, &mut buf),

            Node::Regexp(stat) => {
                handle_vector_of_nodes(stat.parts, &mut buf);
                handle_optional_node(&stat.options, &mut buf)
            }

            Node::Rescue(stat) => {
                handle_vector_of_nodes(stat.rescue_bodies, &mut buf);
                handle_optional_node(&stat.else_, &mut buf);
                handle_optional_node(&stat.else_, &mut buf)
            }
            Node::RescueBody(stat) => {
                handle_optional_node(&stat.body, &mut buf);
                handle_optional_node(&stat.exc_var, &mut buf);
                handle_optional_node(&stat.exc_list, &mut buf)
            }

            Node::Return(stat) => handle_vector_of_nodes(stat.args, &mut buf),

            Node::Send(stat) => {
                match parse_send(stat.clone()) {
                    SendTypes::ParamsPermit => {}
                    SendTypes::ParamsRequire => {}
                    SendTypes::ParamsRequirePermit => {}
                    _ => {
                        // method_calls.insert(stat.method_name);
                        handle_vector_of_nodes(stat.args, &mut buf);
                        handle_optional_node(&stat.recv, &mut buf)
                    }
                }
            }

            Node::Splat(stat) => handle_optional_node(&stat.value, &mut buf),

            Node::Undef(stat) => handle_vector_of_nodes(stat.names, &mut buf),
            Node::UnlessGuard(stat) => buf.push_back(stat.cond),
            Node::Until(stat) => {
                buf.push_back(stat.cond);
                handle_optional_node(&stat.body, &mut buf);
            }
            Node::UntilPost(stat) => {
                buf.push_back(stat.cond);
                buf.push_back(stat.body);
            }

            Node::When(stat) => {
                handle_vector_of_nodes(stat.patterns, &mut buf);
                handle_optional_node(&stat.body, &mut buf)
            }

            Node::While(stat) => {
                buf.push_back(stat.cond);
                handle_optional_node(&stat.body, &mut buf)
            }
            Node::WhilePost(stat) => {
                buf.push_back(stat.cond);
                buf.push_back(stat.body)
            }

            Node::Yield(stat) => handle_vector_of_nodes(stat.args, &mut buf),

            _ => {}
        }
    }
    MethodDetails {
        name: method_name,
        args,
        params,
        headers,
        instance_varaibles,
        method_calls,
        renders,
        local_varaibles,
    }
}

// track all the information going down about if it is params, require, permit
// 1. get down to the bottom to find if params is going to be used
// 2. check whether or not permit and require are in the correct order
// 3. parse the given indexs

fn parse_send(stat: nodes::Send) -> SendTypes {
    let mut require = false;
    let mut permit = false;
    let mut params = false;
    if let Some(temp) = stat.recv {
        if stat.method_name == "require" {
            require = true;
        } else if stat.method_name == "permit" {
            permit = true;
        } else {
            return SendTypes::Invalid;
        }
        let mut buf = VecDeque::new();
        buf.push_back(*temp);
        while let Some(temp) = buf.pop_front() {
            match temp {
                Node::Send(stat) => {
                    if stat.method_name == "require" {
                        if permit {
                            require = true;
                        } else {
                            return SendTypes::Invalid;
                        }
                    } else if stat.method_name == "permit" {
                        if require {
                            return SendTypes::Invalid;
                        }
                        permit = true;
                    } else if stat.method_name == "params" {
                        params = true;
                    } else {
                        return SendTypes::Invalid;
                    }
                    if let Some(recv) = stat.recv {
                        buf.push_back(*recv);
                    }
                }
                _ => {}
            }
        }
    }
    if params {
        if require {
            if permit {
                return SendTypes::ParamsRequirePermit;
            } else {
                return SendTypes::ParamsRequire;
            }
        } else if permit {
            return SendTypes::ParamsPermit;
        }
    }

    SendTypes::Invalid
}

fn params_index(stat: Index) -> Option<Vec<String>> {
    let mut params_found = false;
    let mut buf = VecDeque::new();
    buf.push_back(*stat.recv.clone());
    let mut depth = 0;
    let mut data: Vec<String> = stat
        .indexes
        .iter()
        .map(|x| utils::parse_node_str(x))
        .collect();
    while let Some(temp) = buf.pop_front() {
        depth += 1;
        match temp {
            Node::Send(stat) => {
                if stat.method_name == "params" {
                    params_found = true;
                }
            }
            Node::Index(stat) => {
                buf.push_back(*stat.recv);
                for element in stat.indexes {
                    buf.push_back(element);
                }
            }
            _ => {
                let value = parse_node_str(&temp);
                if value != "unknown".to_owned() {
                    data.push(value);
                }
            }
        }
    }

    if params_found {
        if depth > 1 {
            data.reverse();
            Some(vec![data.join(":")])
        } else {
            Some(data)
        }
    } else {
        None
    }
}

#[cfg(test)]
mod params_tests {
    use lib_ruby_parser::{Node, Parser};
    use pretty_assertions::assert_eq;

    use crate::params::{parse_send, SendTypes};

    use super::create_method_details;

    fn helper(input: &str) -> Box<lib_ruby_parser::Node> {
        Box::new(
            Parser::new(input.as_bytes(), Default::default())
                .do_parse()
                .ast
                .unwrap(),
        )
    }
    fn param_helper(input: &str) -> String {
        let mut results = create_method_details(
            Box::new(
                Parser::new(input.as_bytes(), Default::default())
                    .do_parse()
                    .ast
                    .unwrap(),
            ),
            "tasdf".to_string(),
            Vec::new(),
        )
        .params
        .into_iter()
        .collect::<Vec<String>>();
        results.sort();
        return results.join(", ");
    }

    fn header_helper(input: &str) -> String {
        let temp = helper(input);
        // println!("{:#?}", *temp);
        let mut results = create_method_details(temp, "".to_string(), Vec::new()).headers;
        results.sort();
        return format!("{:?}", results);
    }

    fn method_call_helper(input: &str) -> String {
        // let temp = helper(input);
        // // println!("{:#?}", *temp);
        // let mut results = search_for_param(temp)
        //     .method_calls
        //     .into_iter()
        //     .collect::<Vec<String>>();
        // results.sort();
        // return results.join(", ");
        return "".to_string();
    }

    #[test]
    fn send_method() {
        assert_eq!(param_helper("render 'show'"), "");
    }

    #[test]
    fn params_without_any_index() {
        assert_eq!(param_helper("params"), "");
    }

    #[test]
    fn params_index() {
        assert_eq!(param_helper("params[:id]"), "id");
    }

    #[test]
    fn params_index_string() {
        assert_eq!(param_helper("params['dogs']"), "dogs");
    }

    #[test]
    fn params_index_multiple_string() {
        assert_eq!(param_helper("params['dogs', 'pizza']"), "dogs, pizza");
    }

    #[test]
    fn params_double_index_string() {
        /**
           index
               - index
           - value
        */
        assert_eq!(param_helper("params['cat']['dogs']"), "cat:dogs");
    }

    #[test]
    fn params_require() {
        assert_eq!(
            param_helper("event_type = params.require(:issue_event_type_name)"),
            "issue_event_type_name"
        );
    }

    #[test]
    fn params_permit() {
        assert_eq!(param_helper("event_type = params.permit(:pizza)"), "pizza");
    }

    #[test]
    fn params_permit_in_array() {
        assert_eq!(
            param_helper("event_type = params.permit([:pizza])"),
            "pizza"
        );
    }

    #[test]
    fn params_permit_array_type() {
        assert_eq!(
            param_helper("event_type = params.permit(:pizza => [])"),
            "pizza[]"
        );
    }

    #[test]
    fn params_permit_object_type() {
        assert_eq!(
            param_helper("event_type = params.permit(:pizza => {})"),
            "pizza{}"
        );
    }

    #[test]
    fn params_permit_complex() {
        assert_eq!(
            param_helper("event_type = params.permit(:pizza => [], :dog => {}, :foobar)"),
            "pizza[], dog{}, foobar"
        );
    }

    #[test]
    fn params_require_permit() {
        assert_eq!(
            param_helper("event_type = params.require(:issue_event_type_name).permit(:dogs)"),
            "issue_event_type_name, dogs"
        );
    }

    #[test]
    fn params_send() {
        assert_eq!(
            param_helper("@results = query.foo(params[:issue_event_type_name])"),
            "issue_event_type_name"
        );
    }

    #[test]
    fn params_require_complex() {
        assert_eq!(
            param_helper(
                " create_details =  {
            :project_key => params.require(:project_key),
            :issue_type_id => params.require(:issue_type_id),
            :title_field_key => p[:title_field_key],
            :description_field_key => p[:description_field_key],
            :title => p[:title],
            :description => p[:description]
          }"
            ),
            "issue_type_id, project_key"
        );
    }

    #[test]
    fn params_if() {
        assert_eq!(
            param_helper(
                "if params[:id]
                    @results = params[:cat]
                end"
            ),
            "cat, id"
        );
    }

    // #[test]
    // fn headers_index() {
    //     assert_eq!(header_helper("headers['hello']"), "[(\"hello\", \"\")]");
    // }

    // #[test]
    // fn request_headers() {
    //     assert_eq!(
    //         header_helper("request.headers['hello']"),
    //         "[(\"hello\", \"\")]"
    //     );
    // }

    // #[test]
    // fn headers_assignment() {
    //     assert_eq!(
    //         header_helper("headers['hello'] = 20"),
    //         "[(\"hello\", \"20\")]"
    //     );
    // }

    #[test]
    fn method_call() {
        assert_eq!(
            method_call_helper("process_jwt cookie"),
            "cookie, process_jwt"
        );
    }

    #[test]
    fn method_call_not_render() {
        assert_eq!(method_call_helper("render json: foo"), "");
    }

    #[test]
    fn local_varaible_access_count() {
        let input = "
            a = 1
            b
            c = 2
            puts c
        ";
        let temp = helper(input);
        // println!("{:#?}", *temp);
        let results = create_method_details(temp, "".to_string(), Vec::new()).local_varaibles;

        assert_eq!(results.get("a"), Some(&0));
        assert_eq!(results.get("b"), None);
        assert_eq!(results.get("c"), Some(&1));
    }
    mod param_send_type {
        use super::*;
        use pretty_assertions::assert_eq;

        fn send_helper_tester(input: &str, check: SendTypes) {
            let temp = *helper(input);
            if let Node::Send(value) = temp {
                assert_eq!(parse_send(value), check);
            } else {
                assert_eq!(true, false, "input wassn't a send node");
            }
        }

        #[test]
        fn require_single() {
            send_helper_tester("params.require(:asdf)", SendTypes::ParamsRequire);
        }

        #[test]
        fn permit_single() {
            send_helper_tester("params.permit(:asdf)", SendTypes::ParamsPermit);
        }

        #[test]
        fn require_params_correct_order() {
            send_helper_tester(
                "params.require(:asdf).permit(:asdf)",
                SendTypes::ParamsRequirePermit,
            );
        }

        #[test]
        fn params_require_wrong_order() {
            send_helper_tester("params.permit(:asdf).require(:asdf)", SendTypes::Invalid);
        }
    }

    // #[test]
    // fn test_cat() {
    //     let node = helper("params.require(:issue_event_type_name).permit(:dogs)");
    //     // println!("{:?}", search_send_for_method(&node));
    //     assert_eq!(true, false);
    // }

    // TODO: work out if it is possible to write a integeration test as the order of
    // the fields will keep on changing

    // #[test]
    // fn full_funtional_test() {
    //     let actual = search_for_param(helper(
    //         "
    //     p = params.permit(:user_id, :start, :id, :limit)
    //     @pizza = @pizza.negative if p[:negative]

    //     limit = p[:limit] || 1000
    //     @pizza = @pizza.order(:id).limit(limit)
    //     ",
    //     ));

    //     let mut params = HashSet::new();
    //     params.insert("user_id".to_owned());
    //     params.insert("start".to_owned());
    //     params.insert("id".to_owned());
    //     params.insert("limit".to_owned());

    //     let mut method_calls = HashSet::new();
    //     method_calls.insert("order".to_owned());
    //     method_calls.insert("negative".to_owned());
    //     method_calls.insert("limit".to_owned());

    //     let mut headers = Vec::new();

    //     let mut instance_varaibles = HashSet::new();
    //     instance_varaibles.insert("@pizza".to_owned());
    //     assert_eq!(
    //         actual,
    //         MethodDetails {
    //             params,
    //             headers,
    //             instance_varaibles,
    //             method_calls,
    //             renders: Vec::new(),
    //             local_varaibles: Vec::new(),
    //         }
    //     );
    // }
}
