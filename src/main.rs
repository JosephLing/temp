use std::path::PathBuf;

use argh::FromArgs;
use rts::compute;

fn debug_default() -> bool {
    false
}

#[derive(FromArgs)]
/// Parse Ruby on Rails project to produce api docs
struct RtsCmd {
    /// directory of the ruby on rails project
    #[argh(positional)]
    root: PathBuf,

    /// turn on debug mode
    #[argh(option, default = "debug_default()")]
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd: RtsCmd = argh::from_env();
    let app_data = compute(&cmd.root)?;

    if cmd.debug {
        println!("--- Controllers ---");
        for con in app_data.controllers.values() {
            println!("[{:?}] {} < {}", con.module, con.name, con.parent);
            for include in &con.include {
                println!("#include {}", include);
            }
            for (kind, action) in &con.actions {
                println!("#{:?} {}", kind, action);
            }
            for method in &con.get_own_methods() {
                println!("- {}", method.name);
            }
            for method in &con.get_inherited_methods(&app_data) {
                println!("> {}", method.name);
            }
            for method in &con.get_included_methods(&app_data) {
                println!("+ {}", method.name);
            }
            println!();
        }

        println!("--- Helpers ---");
        for hel in app_data.helpers.values() {
            println!("{}", hel.name);
            for method in &hel.methods {
                println!("- {}", method.name);
            }
            println!();
        }

        println!("--- Concerns ---");
        for con in app_data.concerns.values() {
            println!("{}", con.name);
            for method in &con.methods {
                println!("- {}", method.name);
            }
            println!();
        }

        println!("--- Views ---");
        for (controller, value) in &app_data.views {
            for (action, view) in value {
                println!("{}#{}\n\t{:?}", controller, action, view.response);
            }
        }
    }

    println!("--- Routes ---");
    for route in app_data.routes.values() {
        println!("{}", route);
        print!("@ params = ");
        match route.get_params(&app_data) {
            Ok(p) => println!("{:?}", p),
            Err(err) => println!("{}", err),
        }

        if let Ok(p) = route.get_view(&app_data) {
            println!("Response: {:?}", p)
        }
    }

    Ok(())
}
