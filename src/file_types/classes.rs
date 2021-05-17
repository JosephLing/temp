#[derive(Debug)]
pub struct Controller {
    pub name: String,
    pub parent: String,
    pub methods: Vec<MethodDetails>,
    pub actions: Vec<(ActionKinds, String)>,
    pub include: Vec<String>,
    pub module: Option<String>,
    // ignoring requires for now
}