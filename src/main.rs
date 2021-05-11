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
    compute(&cmd.root)?;
    Ok(())
}
