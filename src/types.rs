use std::collections::{HashMap, HashSet};

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
    pub renders: Vec<String>,                     // TODO: implement this one
}
#[derive(Debug)]
pub enum ActionKinds{
    BeforeAction,
    AroundAction,
    RescueFrom,
    Custom(String)
}

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

#[derive(Debug)]
pub struct HelperModule {
    pub name: String,
    pub methods: Vec<MethodDetails>,
}

#[derive(Debug)]
pub struct Concern {
    pub name: String,
    pub methods: Vec<MethodDetails>,
    pub actions: Vec<String>, // TODO: work out what this looks like
}

#[derive(Debug)]
pub struct AppData {
    pub concerns: HashMap<String, Concern>,
    pub helpers: HashMap<String, HelperModule>,
    pub controllers: HashMap<String, Controller>,
}

impl Controller {
    pub fn get_own_methods(&self) -> Vec<MethodDetails> {
        return self.methods.clone();
    }

    pub fn get_inherited_methods(&self, app_data: &AppData) -> Vec<MethodDetails> {
        match app_data.controllers.get(&self.parent) {
            Some(parent_controller) => parent_controller.get_all_methods(app_data),
            None => Vec::new(),
        }
    }

    pub fn get_all_methods(&self, app_data: &AppData) -> Vec<MethodDetails> {
        let mut methods = self.get_own_methods();
        methods.append(&mut self.get_inherited_methods(app_data));
        return methods;
    }
}
