use crate::config::{Config, Target, MergeTarget, OtaTarget, OutputFormat};
use crate::firmware::{self, FirmwareImage};
use super::context::BuildContext;
use super::error::RecipeError;

pub fn execute_target(config: &Config, target_name: &str, ctx: &BuildContext) -> Result<(), RecipeError> {
    let target = config
        .targets
        .get(target_name)
        .ok_or_else(|| RecipeError::TargetNotFound(target_name.to_owned()))?;

    println!("[{}] Building...", target_name);

    match target {
        Target::Merge(t) => execute_merge(t, target_name, config, ctx),
        Target::Ota(t) => execute_ota(t, target_name, config, ctx),
    }
}


fn execute_merge(target: &MergeTarget, target_name: &str, config: &Config, ctx: &BuildContext) -> Result<(), RecipeError> {
    let bootloader_path = resolve_bootloader(&target.bootloader, config, ctx)?;

    let app_path = ctx.resolve_path(&target.app_file);

    if !bootloader_path.exists() {
        return Err(RecipeError::InputNotFound(bootloader_path));
    }
    if !app_path.exists() {
        return Err(RecipeError::InputNotFound(app_path));
    }

    println!("  Loading bootloader: {}", bootloader_path.display());
    let mut image = firmware::ihex::read(&bootloader_path)?;

    println!("  Loading app: {}", app_path.display());
    let app = firmware::ihex::read(&app_path)?;

    image.merge(&app)?;

    std::fs::create_dir_all(&ctx.output_dir)?;

    let output_name = target
        .output_name
        .as_deref()
        .unwrap_or(target_name);
    let extension = target.output_format.extension();
    let output_filename = format!("{}.{}", output_name, extension);
    let output_path = ctx.output_path(&output_filename);

    println!("  Writing: {}", output_path.display());
    match target.output_format {
        OutputFormat::Hex => firmware::ihex::write(&image, &output_path)?,
        OutputFormat::Bin => firmware::binary::write(&image, &output_path, target.fill_byte)?,
        OutputFormat::Srec => {
            return Err(RecipeError::BuildFailed {
                name: target_name.to_string(),
                reason: "SREC format not yet supported".to_string(),
            });
        }
    }

    println!("[{}] Done", target_name);
    Ok(())

}

fn execute_ota(
    target: &OtaTarget,
    target_name: &str,
    config: &Config,
    ctx: &BuildContext,
) -> Result<(), RecipeError> {
    // TODO
    println!("[{}] OTA packaging not yet implemented", target_name);
    Ok(())
}

fn resolve_bootloader(
    reference: &str,
    config: &Config,
    ctx: &BuildContext,
) -> Result<std::path::PathBuf, RecipeError> {
    let bl = config
        .bootloaders
        .get(reference)
        .ok_or_else(|| RecipeError::BootloaderNotFound(reference.to_owned()))?;

    if let Some(path) = bl.file.clone() {
        return Ok(ctx.resolve_path(path.as_path()));
    }

    Err(RecipeError::BootloaderFileNotSpecified(reference.to_owned()))
    // Ok(ctx.resolve_path())
}