use std::path::Path;
use anyhow::{Context, Result};
use baker_rs::cli::{self, Command};
use baker_rs::config::{self, Config};
use baker_rs::recipe::RecipeBuilder;


fn main() -> Result<()> {
    let cli = cli::parse();

    env_logger::Builder::new()
        .filter_level(cli.log_level())
        .format_timestamp(None)
        .init();

    match cli.command {
        Some(Command::Build { targets }) => {
            let cfg = Config::from_file(&cli.config).context("failed to load config file")?;
            let base_dir = cli.config.parent().unwrap_or(Path::new("."));
            let builder = RecipeBuilder::new(&cfg, base_dir);

            let resolved = cfg.resolve_targets(&targets)?;
            println!("Building targets: {:?}", resolved);

            let recipes = builder.build_batch(&resolved)?;
            for recipe in recipes {
                recipe.validate()?;
                recipe.cook()?;
            }
        }
        Some(Command::List) => {
            let cfg = Config::from_file(&cli.config).context("failed to load config file")?;

            println!("Project: {}\n", cfg.project.name);
            println!("Targets:");
            for (name, target) in &cfg.targets {
                let desc = target.description().unwrap_or("-");
                let type_str = match target {
                    config::Target::Merge(_) => "merge",
                    config::Target::Pack(_) => "pack",
                    config::Target::Convert(_) => "convert",
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
