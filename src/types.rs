use crate::routes::Request;
use std::collections::{HashMap, HashSet, VecDeque};

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
pub enum ActionKinds {
    BeforeAction,
    AroundAction,
    RescueFrom,
    Custom(String),
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
    pub actions: Vec<(ActionKinds, String)>,
}

#[derive(Debug)]
pub struct AppData {
    pub concerns: HashMap<String, Concern>,
    pub helpers: HashMap<String, HelperModule>,
    pub controllers: HashMap<String, Controller>,
    pub routes: HashMap<String, Request>,
    pub views: HashMap<String, HashMap<String, View>>,
}

#[derive(Debug)]
pub enum ViewType {
    Jbuilder,
    Jb,
}
// response is a vector for conditional responses
#[derive(Debug)]
pub struct View {
    pub controller: String,
    pub method: String,
    pub response: Vec<String>,
    pub view_type: ViewType,
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

    pub fn get_included_methods(&self, app_data: &AppData) -> Vec<MethodDetails> {
        let mut methods: Vec<MethodDetails> = Vec::new();
        for included in &self.include {
            let mut include_found = false;
            match app_data.concerns.get(included) {
                Some(con) => {
                    methods.append(&mut con.methods.clone());
                    include_found = true;
                }
                None => {}
            }
            match app_data.helpers.get(included) {
                Some(hel) => {
                    methods.append(&mut hel.methods.clone());
                    include_found = true;
                }
                None => {}
            }
            if !include_found {
                println!("WARNING: Include {} not found for {}", included, self.name);
            }
        }
        return methods;
    }

    pub fn get_all_methods(&self, app_data: &AppData) -> Vec<MethodDetails> {
        let mut methods = self.get_own_methods();
        methods.append(&mut self.get_inherited_methods(app_data));
        methods.append(&mut self.get_included_methods(app_data));
        return methods;
    }

    pub fn get_method_by_name(&self, name: &str, app_data: &AppData) -> Option<MethodDetails> {
        for method in self.get_all_methods(app_data) {
            if method.name == name {
                return Some(method);
            }
        }
        return None;
    }

    pub fn get_method_params(&self, method: &MethodDetails, app_data: &AppData) -> HashSet<String> {
        let mut params = method.params.clone();
        for (sub_name, _) in &method.method_calls {
            if let Some(sub) = self.get_method_by_name(sub_name, app_data) {
                // Currently we can't distinguish between 
                //   def has_permission?(permission)
                //     @user.has_permission?(permission)
                //   end
                if sub.method_calls != method.method_calls && method.args != sub.args{
                    params.extend(self.get_method_params(&sub, app_data));
                }
            } else {
                println!(
                    "WARNING: no details found for {} in controller {}",
                    sub_name, self.name
                );
            }
        }
        return params;
    }
}
