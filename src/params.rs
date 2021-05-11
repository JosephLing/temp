use crate::utils::{self, parse_node_str, parse_optional_name};
use std::collections::{HashMap, HashSet, VecDeque};

use lib_ruby_parser::{
    nodes::{self, Index},
    Node,
};

#[derive(Debug, PartialEq, Clone)]
pub struct MethodDetails {
    pub params: HashSet<String>,
    pub headers: Vec<(String, String)>,
    pub instance_varaibles: HashSet<String>,
    pub local_varaibles: HashMap<String, usize>,
    pub method_calls: HashSet<String>,
    pub renders: Vec<(String, String)>,
}

fn search_for_param_in_list(statements: Vec<Node>, buf: &mut VecDeque<Box<Node>>) {
    for stat in statements {
        buf.push_back(Box::new((stat).clone()));
    }
}

fn optional_thing(body: &Option<Box<Node>>, buf: &mut VecDeque<Box<Node>>) {
    if let Some(body) = body {
        buf.push_back((*body).clone());
    }
}

// doesn't support inline methods and singleton classes
// search for: param and headers (ignores payload)
pub fn search_for_param(statement: Box<Node>) -> MethodDetails {
    let mut params = HashSet::new();
    let mut headers: Vec<(String, String)> = Vec::new();
    let mut instance_varaibles: HashSet<String> = HashSet::new();
    let mut method_calls: HashSet<String> = HashSet::new();
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

            Node::Array(stat) => search_for_param_in_list(stat.elements, &mut buf),
            Node::ArrayPattern(stat) => search_for_param_in_list(stat.elements, &mut buf),
            Node::ArrayPatternWithTail(stat) => search_for_param_in_list(stat.elements, &mut buf),

            Node::Begin(stat) => search_for_param_in_list(stat.statements, &mut buf),

            // note: ignore optional elements of block here
            Node::Block(stat) => optional_thing(&stat.body, &mut buf),
            Node::BlockPass(stat) => buf.push_back(stat.value),

            // Node::Case(stat) => {}
            // Node::CaseMatch(stat) => {}
            // Node::Casgn(stat) => {}
            // Node::Cbase(stat) => {}
            Node::Const(stat) => optional_thing(&stat.scope, &mut buf),

            Node::ConstPattern(stat) => buf.push_back(stat.pattern),

            Node::CSend(stat) => search_for_param_in_list(stat.args, &mut buf),

            // We don't currently use @@var style so not handling it
            // Node::Cvar(stat) => {}
            // Node::Cvasgn(stat) => {}
            Node::Defined(stat) => buf.push_back(stat.value),

            Node::Dstr(stat) => search_for_param_in_list(stat.parts, &mut buf),
            Node::Dsym(stat) => search_for_param_in_list(stat.parts, &mut buf),

            Node::EFlipFlop(stat) => {
                optional_thing(&stat.left, &mut buf);
                optional_thing(&stat.right, &mut buf)
            }

            Node::Ensure(stat) => {
                optional_thing(&stat.ensure, &mut buf);
                optional_thing(&stat.body, &mut buf)
            }

            Node::Erange(stat) => {
                optional_thing(&stat.left, &mut buf);
                optional_thing(&stat.right, &mut buf)
            }

            Node::FindPattern(stat) => search_for_param_in_list(stat.elements, &mut buf),

            Node::For(stat) => {
                buf.push_back(stat.iterator);
                buf.push_back(stat.iteratee);
                optional_thing(&stat.body, &mut buf);
            }

            // global vars $var = 1
            // Node::Gvar(stat) => {}
            // Node::Gvasgn(stat) => {}
            Node::Hash(stat) => search_for_param_in_list(stat.pairs, &mut buf),
            Node::HashPattern(stat) => search_for_param_in_list(stat.elements, &mut buf),

            Node::If(stat) => {
                buf.push_back(stat.cond);
                optional_thing(&stat.if_true, &mut buf);
                optional_thing(&stat.if_false, &mut buf)
            }
            Node::IfGuard(stat) => buf.push_back(stat.cond),
            Node::IFlipFlop(stat) => {
                optional_thing(&stat.left, &mut buf);
                optional_thing(&stat.right, &mut buf);
            }
            Node::IfMod(stat) => {
                buf.push_back(stat.cond);
                optional_thing(&stat.if_true, &mut buf);
                optional_thing(&stat.if_false, &mut buf)
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
                        search_for_param_in_list(send.args, &mut buf);
                        optional_thing(&send.recv, &mut buf);
                    }
                } else {
                    buf.push_back(stat.recv);
                    search_for_param_in_list(stat.indexes, &mut buf);
                }
            }

            Node::InPattern(stat) => {
                buf.push_back(stat.pattern);
                optional_thing(&stat.guard, &mut buf);
                optional_thing(&stat.body, &mut buf)
            }

            Node::Irange(stat) => {
                optional_thing(&stat.left, &mut buf);
                optional_thing(&stat.right, &mut buf);
            }

            // TODO: do we want to keep track of instance varaibles?? seems like it is isn't necessary
            // Node::Ivar(stat) => stat.name,
            Node::Ivasgn(stat) => {
                instance_varaibles.insert(stat.name);

                optional_thing(&stat.value, &mut buf)
            }

            Node::Kwargs(stat) => search_for_param_in_list(stat.pairs, &mut buf),
            Node::KwBegin(stat) => search_for_param_in_list(stat.statements, &mut buf),
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
                optional_thing(&stat.value, &mut buf)
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
            Node::MatchRest(stat) => optional_thing(&stat.name, &mut buf),
            Node::MatchWithLvasgn(stat) => {
                buf.push_back(stat.re);
                buf.push_back(stat.value)
            }

            Node::Mlhs(stat) => search_for_param_in_list(stat.items, &mut buf),

            Node::Next(stat) => search_for_param_in_list(stat.args, &mut buf),

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

            Node::Postexe(stat) => optional_thing(&stat.body, &mut buf),
            Node::Preexe(stat) => optional_thing(&stat.body, &mut buf),
            Node::Procarg0(stat) => search_for_param_in_list(stat.args, &mut buf),

            Node::Regexp(stat) => {
                search_for_param_in_list(stat.parts, &mut buf);
                optional_thing(&stat.options, &mut buf)
            }

            Node::Rescue(stat) => {
                search_for_param_in_list(stat.rescue_bodies, &mut buf);
                optional_thing(&stat.else_, &mut buf);
                optional_thing(&stat.else_, &mut buf)
            }
            Node::RescueBody(stat) => {
                optional_thing(&stat.body, &mut buf);
                optional_thing(&stat.exc_var, &mut buf);
                optional_thing(&stat.exc_list, &mut buf)
            }

            Node::Return(stat) => search_for_param_in_list(stat.args, &mut buf),

            Node::Send(stat) => {
                // permit -> require -> params
                // require -> params
                // permit -> params (permit([...])) hash string to data type
                // params
                if let Some(recv) = stat.recv.clone() {
                    if let Node::Send(send_param) = *recv {
                        if send_param.method_name == "params" {
                            for arg in stat.args {
                                match arg {
                                    Node::Sym(value) => {
                                        params.insert(value.name.to_string_lossy());
                                    }
                                    _ => {}
                                }
                            }
                        } else if send_param.method_name == "headers" {
                            println!("{:?}", send_param);
                        } else {
                            method_calls.insert(stat.method_name);
                            search_for_param_in_list(stat.args, &mut buf);
                            optional_thing(&stat.recv, &mut buf)
                        }
                    } else {
                        method_calls.insert(stat.method_name);
                        search_for_param_in_list(stat.args, &mut buf);
                        optional_thing(&stat.recv, &mut buf)
                    }
                } else {
                    method_calls.insert(stat.method_name);
                    search_for_param_in_list(stat.args, &mut buf);
                    optional_thing(&stat.recv, &mut buf)
                }
            }

            Node::Splat(stat) => optional_thing(&stat.value, &mut buf),

            Node::Undef(stat) => search_for_param_in_list(stat.names, &mut buf),
            Node::UnlessGuard(stat) => buf.push_back(stat.cond),
            Node::Until(stat) => {
                buf.push_back(stat.cond);
                optional_thing(&stat.body, &mut buf);
            }
            Node::UntilPost(stat) => {
                buf.push_back(stat.cond);
                buf.push_back(stat.body);
            }

            Node::When(stat) => {
                search_for_param_in_list(stat.patterns, &mut buf);
                optional_thing(&stat.body, &mut buf)
            }

            Node::While(stat) => {
                buf.push_back(stat.cond);
                optional_thing(&stat.body, &mut buf)
            }
            Node::WhilePost(stat) => {
                buf.push_back(stat.cond);
                buf.push_back(stat.body)
            }

            Node::Yield(stat) => search_for_param_in_list(stat.args, &mut buf),

            _ => {}
        }
    }
    MethodDetails {
        params,
        headers,
        instance_varaibles,
        method_calls,
        renders,
        local_varaibles,
    }
}

fn params_send(stat: nodes::Send) {
    /*

    Allowed:
    params.require().permit()

    params.require()

    paramrs.permit()

    NOT allowed:
    params.permit().require()

     */
}

fn params_permit(stat: Node) {
    /*
    params.permit([:opportunity_id, :status => []])

    params.permit(:opportunity_id, :status => [])

    params.permit(:status => {})

    params.permit([:status => {}])
    */
}

fn params_require(stat: Node) {}

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
    use lib_ruby_parser::Parser;
    use pretty_assertions::assert_eq;

    use super::search_for_param;

    fn helper(input: &str) -> Box<lib_ruby_parser::Node> {
        Box::new(
            Parser::new(input.as_bytes(), Default::default())
                .do_parse()
                .ast
                .unwrap(),
        )
    }
    fn param_helper(input: &str) -> String {
        let mut results = search_for_param(Box::new(
            Parser::new(input.as_bytes(), Default::default())
                .do_parse()
                .ast
                .unwrap(),
        ))
        .params
        .into_iter()
        .collect::<Vec<String>>();
        results.sort();
        return results.join(", ");
    }

    fn header_helper(input: &str) -> String {
        let temp = helper(input);
        // println!("{:#?}", *temp);
        let mut results = search_for_param(temp).headers;
        results.sort();
        return format!("{:?}", results);
    }

    fn method_call_helper(input: &str) -> String {
        let temp = helper(input);
        // println!("{:#?}", *temp);
        let mut results = search_for_param(temp)
            .method_calls
            .into_iter()
            .collect::<Vec<String>>();
        results.sort();
        return results.join(", ");
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
        let results = search_for_param(temp).local_varaibles;

        assert_eq!(results.get("a"), Some(&0));
        assert_eq!(results.get("b"), None);
        assert_eq!(results.get("c"), Some(&1));
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
