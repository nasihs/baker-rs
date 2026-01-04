use clap::{CommandFactory, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "baker")]
#[command(bin_name = "baker")]
#[command(about = "Build Automation Kit for Embedded Release")]
#[command(version)]
pub struct Cli {
    #[arg(short, long, default_value = "baker.toml")]
    pub config: PathBuf,

    /// Increase verbosity (-v: info, -vv: debug, -vvv: trace)
    #[arg(short, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Option<Command>,
}

impl Cli {
    /// Get the log level based on verbosity
    pub fn log_level(&self) -> log::LevelFilter {
        match self.verbose {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            2 => log::LevelFilter::Debug,
            _ => log::LevelFilter::Trace,
        }
    }
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

pub fn print_help() {
    Cli::command().print_help().unwrap();
}