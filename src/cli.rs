use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "baker")]
#[command(about = "Build Automation Kit for Embedded Releasa")]
#[command(version)]
pub struct Cli {
    #[arg(short, long, default_value = "baker.toml")]
    pub config: PathBuf,

    #[arg(short, long)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Build specified targets
    Build {
        /// Target names to build
        targets: Vec<String>,
    },
    /// List all targets and groups
    List,
    /// Initialize a new baker.toml
    Init,
    /// Show extracted version
    Version,
}

pub fn parse() -> Cli {
    Cli::parse()
}