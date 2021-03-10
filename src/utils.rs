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
        Node::Sym(sym) => sym.name.to_string_lossy(),
        Node::Ivar(ivar) => ivar.name.clone(),
        Node::Str(str) => str.value.to_string_lossy(),
        Node::Int(int) => int.value.clone(),
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
