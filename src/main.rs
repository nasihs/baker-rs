use anyhow::{Context, Result};
use baker_rs::cli::{self, Command};

fn main() -> Result<()> {
    let cli = cli::parse();

    match cli.command {
        Some(Command::Build { targets }) => {
            println!("Building targets: {:?}", targets);
            // TODO: implement
        }
        Some(Command::List) => {
            println!("Listing targets...");
            // TODO
        }
        Some(Command::Init) => {
            println!("Initializing baker.toml...");
        }
        Some(Command::Version) => {
            println!("Extracting version...");
        }
        None => {
            println!("No command specified, buiding default target...");
        }
    }

    Ok(())
}