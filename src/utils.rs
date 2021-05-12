use lib_ruby_parser::Node;

pub fn parse_optional_name(node: &Option<Box<Node>>) -> String {
    if let Some(n) = node {
        parse_name((*n).clone())
    } else {
        "".to_owned()
    }
}

pub fn parse_node_str(node: &Node) -> String {
    match node {
        Node::Send(stat) => {
            if stat.args.is_empty() && stat.recv.is_none() {
                return stat.method_name.clone();
            } else {
                "unknown".to_string()
            }
        }
        Node::Sym(sym) => sym.name.to_string_lossy(),
        Node::Ivar(ivar) => ivar.name.clone(),
        Node::Str(str) => str.value.to_string_lossy(),
        Node::Int(int) => int.value.clone(),
        Node::Array(array) => {
            if array.elements.is_empty() {
                return "[]".to_owned();
            } else {
                format!(
                    "[{}]",
                    array
                        .elements
                        .iter()
                        .map(|x| parse_node_str(x))
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
        Node::Hash(hash) => {
            if hash.pairs.is_empty() {
                return "{}".to_owned();
            } else {
                format!(
                    "{{{}}}",
                    hash.pairs
                        .iter()
                        .map(|x| parse_node_str(x))
                        .collect::<Vec<String>>()
                        .join(",")
                )
            }
        }
        Node::Nil(nil) => "nil".to_owned(),
        Node::Kwargs(kwargs) => kwargs
            .pairs
            .iter()
            .map(|kwarg| parse_node_str(kwarg))
            .collect::<Vec<String>>()
            .join(","),
        Node::Const(node_const_name) => {
            if let Some(scope) = &node_const_name.scope {
                format!(
                    "{}::{}",
                    get_node_name(scope).unwrap(),
                    node_const_name.name
                )
            } else {
                node_const_name.name.to_string()
            }
        }
        Node::Lvar(lvar) => lvar.name.clone(),
        Node::Pair(pair) => {
            format!(
                "{}=>{}",
                parse_node_str(&pair.key),
                parse_node_str(&pair.value)
            )
        }
        Node::Or(or) => format!("{} or {}", parse_node_str(&or.lhs), parse_node_str(&or.rhs)),
        Node::True(_) => "true".to_owned(),
        Node::False(_) => "false".to_owned(),
        // Node::Regexp(_) => "regexp TODO".to_owned(),
        // Node::Dstr(_) => "regexp TODO".to_owned(),
        Node::Index(stat) => format!(
            "{}[{}]",
            parse_node_str(&stat.recv),
            stat.indexes
                .iter()
                .map(|x| parse_node_str(x))
                .collect::<Vec<String>>()
                .join(",")
        ),
        _ => "unknown".to_string(),
    }
}

pub fn parse_name(node: Box<Node>) -> String {
    parse_node_str(&*node)
}

pub fn parse_superclass(node: Option<Box<Node>>) -> String {
    if let Some(boxed_node) = node {
        parse_name(boxed_node)
    } else {
        "".to_string()
    }
}

pub fn get_node_name(name: &Node) -> Result<String, String> {
    match name {
        Node::Const(node_const_name) => {
            if let Some(scope) = &node_const_name.scope {
                Ok(format!(
                    "{}::{}",
                    get_node_name(scope)?,
                    node_const_name.name
                ))
            } else {
                Ok(node_const_name.name.to_string())
            }
        }
        _ => Err("could not get name".to_string()),
    }
}
