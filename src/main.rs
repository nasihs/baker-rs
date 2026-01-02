use anyhow::{Context, Result};
use baker_rs::cli::{self, Command};
use baker_rs::config;

fn main() -> Result<()> {
    let cli = cli::parse();

    // 加载配置
    let cfg = config::load(&cli.config).context("failed to load config")?;

    if cli.verbose {
        println!("Project: {}", cfg.project.name);
        println!("Targets: {:?}", cfg.targets.keys().collect::<Vec<_>>());
        println!("Groups: {:?}", cfg.groups.keys().collect::<Vec<_>>());
    }

    match cli.command {
        Some(Command::Build { targets }) => {
            let resolved = cfg.resolve_targets(&targets)?;
            println!("Building targets: {:?}", resolved);

            for target_name in resolved {
                let target = cfg.targets.get(target_name).unwrap();
                println!("\n[{}]", target_name);
                match target {
                    config::Target::Merge(t) => {
                        println!("  Type: merge");
                        println!("  App: {}", t.app.display());
                        println!("  Bootloader: {}", t.bootloader);
                        println!("  Offset: 0x{:X}", t.app_offset);
                    }
                    config::Target::Ota(t) => {
                        println!("  Type: ota");
                        println!("  Input: {}", t.input.display());
                        println!("  Header: {:?}", t.header);
                    }
                }
            }
        }
        Some(Command::List) => {
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
            let resolved = cfg.resolve_targets(&[])?;
            println!("Building default targets: {:?}", resolved);
        }
    }

    Ok(())
}
