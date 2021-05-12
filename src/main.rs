use std::path::PathBuf;

use argh::FromArgs;
use rts::compute;

#[derive(FromArgs)]
/// Parse Ruby on Rails project to produce api docs
struct RtsCmd {
    /// directory of the ruby on rails project
    #[argh(positional)]
    root: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cmd: RtsCmd = argh::from_env();
    let app_data = compute(&cmd.root)?;

    println!("--- Controllers ---");
    for (_, con) in app_data.controllers {
        println!("[{:?}] {} < {}", con.module, con.name, con.parent);
        for method in con.methods {
            println!("{}", method);
        }
    }

    println!("--- Helpers ---");
    for (_, hel) in app_data.helpers {
        println!("{}", hel.name);
        for method in hel.methods {
            println!("{}", method);
        }
    }

    println!("--- Concerns ---");
    for (_, con) in app_data.concerns {
        println!("{}", con.name);
        for method in con.methods {
            println!("{}", method);
        }
    }
    Ok(())
}
