use std::path::Path;
use anyhow::{Context, Result};
use log::{info};
use baker_rs::cli::{self, Command};
use baker_rs::config;
use baker_rs::recipe::RecipeBuilder;

fn main() -> Result<()> {
    let cli = cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.log_level())
        .format_timestamp(None)
        .init();

    match cli.command {
        Some(Command::Build { targets }) => {
            let cfg = config::load(&cli.config).context("failed to load config")?;
            let base_dir = cli.config.parent().unwrap_or(Path::new("."));
            let builder = RecipeBuilder::new(&cfg, base_dir);

            let resolved = cfg.resolve_targets(&targets)?;
            info!("Building targets: {:?}", resolved);

            for target_name in resolved {
                let recipe = builder.build(target_name)?;
                recipe.validate()?;
                recipe.cook()?;
            }
        }
        Some(Command::List) => {
            let cfg = config::load(&cli.config).context("failed to load config")?;

            println!("Project: {}\n", cfg.project.name);
            println!("Targets:");
            for (name, target) in &cfg.targets {
                let desc = target.description().unwrap_or("-");
                let type_str = match target {
                    config::Target::Merge(_) => "merge",
                    config::Target::Ota(_) => "ota",
                };
                println!("  {:<15} [{}] {}", name, type_str, desc);
            }

            if !cfg.groups.is_empty() {
                println!("\nGroups:");
                for (name, group) in &cfg.groups {
                    println!("  {:<15} -> {:?}", name, group.targets());
                }
            }
        }
        Some(Command::Init) => {
            println!("TODO: Initialize baker.toml");
        }
        Some(Command::Version) => {
            println!("TODO: Extract version");
        }
        None => {
            cli::print_help();
        }
    }

    Ok(())
}
