# Post-Build Hook Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an optional `post_build` field to all target types (merge/pack/convert) that runs an external command with `${VAR}` substitution after the main output is written.

**Architecture:** `PostBuildHook` is deserialized from config and converted to a `HookRunner` at recipe-build time. The runner pre-bakes all baker env vars (including `TARGET`) and expands `OUTPUT_FILE`, `OUTPUT_DIR`, `OUTPUT_NAME` at `run()` time when the output path is known. The template rendering logic is extracted from `RecipeBuilder` into a shared `render` module so both builder and hook can use it.

**Tech Stack:** Rust, `std::process::Command`, `regex`, `serde`, `thiserror`

---

## Branch Setup

This feature is independent of `feature/examples`. Create a new branch from `main`:

```bash
git checkout main
git checkout -b feature/post-build-hook
```

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `src/config/schema.rs` | Modify | Add `PostBuildHook` struct; add `post_build` field to all 3 target structs |
| `src/recipe/error.rs` | Modify | Add `ExternalToolNotFound` and `ExternalToolFailed` error variants |
| `src/recipe/render.rs` | **Create** | Free functions `render_template` and `value_to_string` (extracted from builder) |
| `src/recipe/hook.rs` | **Create** | `HookRunner` struct with `run(output_path)` method |
| `src/recipe/builder.rs` | Modify | Use `render` module; build `HookRunner` from config in `build_merge/pack/convert` |
| `src/recipe/merge.rs` | Modify | Add `hook: Option<HookRunner>` field; call after write |
| `src/recipe/convert.rs` | Modify | Same as merge |
| `src/recipe/pack.rs` | Modify | Same as merge |
| `src/recipe.rs` | Modify | Add `pub(super) mod hook;` |

---

## Task 1: Config Schema + Error Variants

**Files:**
- Modify: `src/config/schema.rs`
- Modify: `src/recipe/error.rs`

### Step 1: Write the failing deserialization test

Add to the bottom of `src/config/schema.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    #[test]
    fn test_post_build_hook_deserialization() {
        let toml_str = r#"
            [project]
            name = "test"
            default = "t1"

            [targets.t1]
            type = "convert"
            input_file = "build/app.hex"

            [targets.t1.post_build]
            command = "jflash.exe"
            args = ["-openprj", "device.jflash", "-open", "${OUTPUT_FILE}"]
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let target = config.targets.get("t1").unwrap();
        let hook = match target {
            Target::Convert(t) => t.post_build.as_ref().unwrap(),
            _ => panic!("expected convert target"),
        };
        assert_eq!(hook.command, "jflash.exe");
        assert_eq!(hook.args[0], "-openprj");
        assert_eq!(hook.args[3], "${OUTPUT_FILE}");
    }

    #[test]
    fn test_post_build_hook_optional() {
        let toml_str = r#"
            [project]
            name = "test"
            default = "t1"

            [targets.t1]
            type = "convert"
            input_file = "build/app.hex"
        "#;
        let config: Config = toml::from_str(toml_str).unwrap();
        let target = config.targets.get("t1").unwrap();
        let hook = match target {
            Target::Convert(t) => &t.post_build,
            _ => panic!("expected convert target"),
        };
        assert!(hook.is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

```bash
cargo test test_post_build_hook_deserialization -- --nocapture
```

Expected: compile error — `post_build` field not found on `ConvertTarget`

- [ ] **Step 3: Implement `PostBuildHook` struct and add fields**

In `src/config/schema.rs`:

1. Add new struct after `OutputFormat`:

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct PostBuildHook {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}
```

2. Add `post_build` field to `MergeTarget`:

```rust
pub struct MergeTarget {
    pub description: Option<String>,
    pub bootloader: String,
    pub app_file: PathBuf,
    #[serde(default = "default_fill_byte")]
    pub fill_byte: u8,
    #[serde(default)]
    pub output_format: OutputFormat,
    pub output_name: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub post_build: Option<PostBuildHook>,  // ADD THIS
}
```

3. Add `post_build` field to `ConvertTarget`:

```rust
pub struct ConvertTarget {
    pub description: Option<String>,
    pub input_file: PathBuf,
    #[serde(default = "default_fill_byte")]
    pub fill_byte: u8,
    #[serde(default)]
    pub output_format: OutputFormat,
    pub output_name: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub post_build: Option<PostBuildHook>,  // ADD THIS
}
```

4. Add `post_build` field to `PackTarget`:

```rust
pub struct PackTarget {
    pub description: Option<String>,
    pub header: String,
    pub app_file: PathBuf,
    pub app_offset: Option<u32>,
    #[serde(default = "default_fill_byte")]
    pub fill_byte: u8,
    pub output_name: Option<String>,
    pub output_dir: Option<PathBuf>,
    pub post_build: Option<PostBuildHook>,  // ADD THIS
}
```

- [ ] **Step 4: Add error variants**

In `src/recipe/error.rs`, add two variants:

```rust
#[error("external tool not found: '{0}' (is it installed and on PATH?)")]
ExternalToolNotFound(String),

#[error("external tool '{command}' failed with exit code {exit_code}: {stderr}")]
ExternalToolFailed {
    command: String,
    exit_code: String,
    stderr: String,
},
```

Note: `exit_code` is `String` (not `Option<i32>`) to make the `Display` impl via thiserror simple — format it as `"<unknown>"` when `Option<i32>` is None (see Task 3 step where the conversion happens).

- [ ] **Step 5: Re-export `PostBuildHook` from `src/config.rs`**

In `src/config.rs`, add `PostBuildHook` to the `pub use schema::` line:

```rust
pub use schema::{
    Bootloader, Config, ConvertTarget, Env, Group, HeaderDef, MergeTarget, PackTarget,
    OutputConfig, OutputFormat, PostBuildHook, Project, Target, VersionConfig, VersionSource,
};
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cargo test test_post_build_hook -- --nocapture
```

Expected: both tests PASS; `cargo build` compiles cleanly

- [ ] **Step 7: Commit**

```bash
git add src/config/schema.rs src/config.rs src/recipe/error.rs
git commit -m "feat(config): add PostBuildHook schema; add external tool error variants"
```

---

## Task 2: Extract Render to Shared Module

The `render_template` and `value_to_string` functions are currently private methods on `RecipeBuilder`. The new `HookRunner` needs them too. Extract them to `src/recipe/render.rs`.

**Files:**
- Create: `src/recipe/render.rs`
- Modify: `src/recipe/builder.rs`
- Modify: `src/recipe.rs`

- [ ] **Step 1: Create `src/recipe/render.rs`**

```rust
use std::collections::HashMap;
use super::RecipeError;

/// Substitute all `${VAR}` placeholders in `template` using `vars`.
/// Returns `Err(MissingVariable)` if any placeholder remains unresolved.
pub(super) fn render_template(
    vars: &HashMap<String, delbin::Value>,
    template: &str,
) -> Result<String, RecipeError> {
    let mut result = template.to_string();

    for (key, value) in vars {
        let placeholder = format!("${{{}}}", key);
        let value_str = value_to_string(value);
        result = result.replace(&placeholder, &value_str);
    }

    let re = regex::Regex::new(r"\$\{([A-Z_][A-Z0-9_.]*)\}").unwrap();
    if let Some(cap) = re.captures(&result) {
        let var_name = cap[1].to_string();
        return Err(RecipeError::MissingVariable(var_name));
    }

    Ok(result)
}

/// Convert a `delbin::Value` to its string representation for template substitution.
pub(super) fn value_to_string(value: &delbin::Value) -> String {
    match value {
        delbin::Value::U8(v)  => v.to_string(),
        delbin::Value::U16(v) => v.to_string(),
        delbin::Value::U32(v) => v.to_string(),
        delbin::Value::U64(v) => v.to_string(),
        delbin::Value::I8(v)  => v.to_string(),
        delbin::Value::I16(v) => v.to_string(),
        delbin::Value::I32(v) => v.to_string(),
        delbin::Value::I64(v) => v.to_string(),
        delbin::Value::String(s) => s.clone(),
        delbin::Value::Bytes(b) => b.iter().map(|byte| format!("{:02X}", byte)).collect::<Vec<_>>().join(""),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(pairs: &[(&str, delbin::Value)]) -> HashMap<String, delbin::Value> {
        pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect()
    }

    #[test]
    fn test_render_dotted_var_name() {
        let env = vars(&[
            ("VER.MAJOR", delbin::Value::U32(2)),
            ("VER.MINOR", delbin::Value::U32(5)),
        ]);
        assert_eq!(
            render_template(&env, "fw_v${VER.MAJOR}.${VER.MINOR}").unwrap(),
            "fw_v2.5"
        );
    }

    #[test]
    fn test_render_missing_variable() {
        let env = vars(&[("VER.MAJOR", delbin::Value::U32(1))]);
        match render_template(&env, "fw_v${VER.MAJOR}_${VER.UNDEFINED}").unwrap_err() {
            RecipeError::MissingVariable(name) => assert_eq!(name, "VER.UNDEFINED"),
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn test_render_no_placeholders() {
        let env = vars(&[("VER.MAJOR", delbin::Value::U32(1))]);
        assert_eq!(
            render_template(&env, "firmware_latest").unwrap(),
            "firmware_latest"
        );
    }
}
```

- [ ] **Step 2: Register the new module in `src/recipe.rs`**

Add `mod render;` (no pub re-export needed — it's `pub(super)` within the recipe module):

```rust
mod error;
mod merge;
mod pack;
mod convert;
mod builder;
mod render;     // ADD THIS

pub use error::RecipeError;
pub use merge::MergeRecipe;
pub use pack::{PackRecipe, BuiltinHeaders};
pub use convert::ConvertRecipe;
pub use builder::RecipeBuilder;

// ... rest unchanged
```

- [ ] **Step 3: Update `builder.rs` to use the shared render module**

Replace the private `render` and `value_to_string` methods on `RecipeBuilder` with calls to the shared module:

1. Remove the `render` associated function from `RecipeBuilder` (the one taking `vars` and `template`).
2. Remove `value_to_string` associated function from `RecipeBuilder`.
3. Update `render_template` method to call `super::render::render_template`:

```rust
fn render_template(&self, template: &str, target_name: &str) -> Result<String, RecipeError> {
    let mut vars = self.env.clone();
    vars.insert("TARGET".to_string(), delbin::Value::String(target_name.to_string()));
    super::render::render_template(&vars, template)
}
```

4. Remove the test module from `builder.rs` (tests moved to `render.rs`).

- [ ] **Step 4: Run all tests**

```bash
cargo test
```

Expected: all tests pass (render tests now live in `render.rs`); no regressions

- [ ] **Step 5: Commit**

```bash
git add src/recipe/render.rs src/recipe/builder.rs src/recipe.rs
git commit -m "refactor(recipe): extract render_template to shared render module"
```

---

## Task 3: HookRunner Implementation

**Files:**
- Create: `src/recipe/hook.rs`
- Modify: `src/recipe.rs` (add `mod hook;`)

- [ ] **Step 1: Write failing tests first**

Create `src/recipe/hook.rs` with just the test module (implementation comes after):

```rust
use std::collections::HashMap;
use std::path::Path;
use super::RecipeError;

pub(super) struct HookRunner {
    pub(super) command: String,
    pub(super) arg_templates: Vec<String>,
    pub(super) vars: HashMap<String, delbin::Value>,
}

impl HookRunner {
    pub(super) fn run(&self, output_path: &Path) -> Result<(), RecipeError> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn make_runner(command: &str, args: &[&str], vars: &[(&str, &str)]) -> HookRunner {
        HookRunner {
            command: command.to_string(),
            arg_templates: args.iter().map(|s| s.to_string()).collect(),
            vars: vars.iter()
                .map(|(k, v)| (k.to_string(), delbin::Value::String(v.to_string())))
                .collect(),
        }
    }

    #[test]
    fn test_hook_runs_successfully() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("fw_1.2.hex");
        std::fs::write(&output_path, b"dummy").unwrap();

        #[cfg(windows)]
        let runner = make_runner("cmd", &["/c", "echo", "ok"], &[]);
        #[cfg(not(windows))]
        let runner = make_runner("sh", &["-c", "echo ok"], &[]);

        runner.run(&output_path).unwrap();
    }

    #[test]
    fn test_hook_expands_output_vars() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("fw_1.2.hex");
        std::fs::write(&output_path, b"dummy").unwrap();

        // Use a command that prints its arg; capture via exit code trick
        // We verify expansion doesn't error (not the printed value)
        #[cfg(windows)]
        let runner = make_runner("cmd", &["/c", "echo", "${OUTPUT_FILE}"], &[]);
        #[cfg(not(windows))]
        let runner = make_runner("sh", &["-c", "echo ${OUTPUT_FILE}"], &[]);

        // Should not return MissingVariable — OUTPUT_FILE is injected by run()
        runner.run(&output_path).unwrap();
    }

    #[test]
    fn test_hook_expands_baker_vars() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("fw_1.2.hex");
        std::fs::write(&output_path, b"dummy").unwrap();

        #[cfg(windows)]
        let runner = make_runner("cmd", &["/c", "echo", "${PROJECT}"], &[("PROJECT", "myapp")]);
        #[cfg(not(windows))]
        let runner = make_runner("sh", &["-c", "echo ${PROJECT}"], &[("PROJECT", "myapp")]);

        runner.run(&output_path).unwrap();
    }

    #[test]
    fn test_hook_command_not_found() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("fw.hex");
        std::fs::write(&output_path, b"dummy").unwrap();

        let runner = make_runner("this_command_absolutely_does_not_exist_xyz123", &[], &[]);
        match runner.run(&output_path).unwrap_err() {
            RecipeError::ExternalToolNotFound(cmd) => {
                assert_eq!(cmd, "this_command_absolutely_does_not_exist_xyz123");
            }
            other => panic!("expected ExternalToolNotFound, got: {other}"),
        }
    }

    #[test]
    fn test_hook_command_fails() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("fw.hex");
        std::fs::write(&output_path, b"dummy").unwrap();

        #[cfg(windows)]
        let runner = make_runner("cmd", &["/c", "exit", "1"], &[]);
        #[cfg(not(windows))]
        let runner = make_runner("sh", &["-c", "exit 1"], &[]);

        match runner.run(&output_path).unwrap_err() {
            RecipeError::ExternalToolFailed { command, .. } => {
                #[cfg(windows)]
                assert_eq!(command, "cmd");
                #[cfg(not(windows))]
                assert_eq!(command, "sh");
            }
            other => panic!("expected ExternalToolFailed, got: {other}"),
        }
    }

    #[test]
    fn test_hook_undefined_variable_in_args() {
        let tmp = TempDir::new().unwrap();
        let output_path = tmp.path().join("fw.hex");
        std::fs::write(&output_path, b"dummy").unwrap();

        // ${UNDEFINED_VAR} is not in vars and not an OUTPUT_* var
        #[cfg(windows)]
        let runner = make_runner("cmd", &["/c", "echo", "${UNDEFINED_VAR}"], &[]);
        #[cfg(not(windows))]
        let runner = make_runner("sh", &["-c", "echo ${UNDEFINED_VAR}"], &[]);

        match runner.run(&output_path).unwrap_err() {
            RecipeError::MissingVariable(name) => assert_eq!(name, "UNDEFINED_VAR"),
            other => panic!("expected MissingVariable, got: {other}"),
        }
    }
}
```

- [ ] **Step 2: Add `tempfile` dev dependency**

In `Cargo.toml`, add to `[dev-dependencies]`:
```toml
tempfile = "3"
```

- [ ] **Step 3: Register module in `src/recipe.rs`**

```rust
mod render;
mod hook;      // ADD THIS
```

- [ ] **Step 4: Run tests to verify they fail**

```bash
cargo test -p baker-rs hook -- --nocapture
```

Expected: compile error (todo!() panics) or test failures for the `todo!()` impl

- [ ] **Step 5: Implement `HookRunner::run()`**

Replace the `todo!()` in `src/recipe/hook.rs` with:

```rust
impl HookRunner {
    pub(super) fn run(&self, output_path: &Path) -> Result<(), RecipeError> {
        // Build expanded vars: clone baker vars, inject OUTPUT_* variables
        let mut vars = self.vars.clone();
        
        let output_dir = output_path.parent().unwrap_or(Path::new("."));
        let output_name = output_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        vars.insert("OUTPUT_FILE".to_string(), delbin::Value::String(output_path.display().to_string()));
        vars.insert("OUTPUT_DIR".to_string(),  delbin::Value::String(output_dir.display().to_string()));
        vars.insert("OUTPUT_NAME".to_string(), delbin::Value::String(output_name.to_string()));

        // Expand ${VAR} in each arg template
        let args: Result<Vec<String>, RecipeError> = self.arg_templates
            .iter()
            .map(|tmpl| super::render::render_template(&vars, tmpl))
            .collect();
        let args = args?;

        println!("  Running post-build hook: {} {}", self.command, args.join(" "));

        let output = std::process::Command::new(&self.command)
            .args(&args)
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    RecipeError::ExternalToolNotFound(self.command.clone())
                } else {
                    RecipeError::Io(e)
                }
            })?;

        if !output.status.success() {
            let exit_code = output.status.code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            return Err(RecipeError::ExternalToolFailed {
                command: self.command.clone(),
                exit_code,
                stderr,
            });
        }

        Ok(())
    }
}
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cargo test hook -- --nocapture
```

Expected: all 6 hook tests PASS

- [ ] **Step 7: Commit**

```bash
git add src/recipe/hook.rs src/recipe.rs Cargo.toml Cargo.lock
git commit -m "feat(recipe): add HookRunner for post-build external commands"
```

---

## Task 4: Wire HookRunner into Builder and Recipes

**Files:**
- Modify: `src/recipe/builder.rs`
- Modify: `src/recipe/merge.rs`
- Modify: `src/recipe/convert.rs`
- Modify: `src/recipe/pack.rs`

- [ ] **Step 1: Add `hook` field to all recipe structs**

In `src/recipe/merge.rs` — add field and call:

```rust
use super::hook::HookRunner;

pub struct MergeRecipe {
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) bootloader_reader: Box<dyn ImageReader>,
    pub(super) app_reader: Box<dyn ImageReader>,
    pub(super) writer: Box<dyn ImageWriter>,
    pub(super) output_path: PathBuf,
    pub(super) hook: Option<HookRunner>,  // ADD THIS
}
```

In `cook()`, after `self.writer.write(&image)?;` and before `Ok(...)`:

```rust
if let Some(hook) = &self.hook {
    hook.run(&self.output_path)?;
}
```

In `src/recipe/convert.rs` — same change:

```rust
use super::hook::HookRunner;

pub struct ConvertRecipe {
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) reader: Box<dyn ImageReader>,
    pub(super) writer: Box<dyn ImageWriter>,
    pub(super) output_path: PathBuf,
    pub(super) hook: Option<HookRunner>,  // ADD THIS
}
```

In `cook()`, after `self.writer.write(&image)?;` and before `Ok(...)`:

```rust
if let Some(hook) = &self.hook {
    hook.run(&self.output_path)?;
}
```

In `src/recipe/pack.rs` — same change:

```rust
use super::hook::HookRunner;

pub struct PackRecipe {
    pub(super) name: String,
    pub(super) description: Option<String>,
    pub(super) app_reader: Box<dyn ImageReader>,
    pub(super) writer: Box<dyn ImageWriter>,
    pub(super) output_path: PathBuf,
    pub(super) header_builder: HeaderBuilder,
    pub(super) hook: Option<HookRunner>,  // ADD THIS
}
```

In `cook()`, after `self.writer.write(&new_image)?;` and before `Ok(...)`:

```rust
if let Some(hook) = &self.hook {
    hook.run(&self.output_path)?;
}
```

- [ ] **Step 2: Build `HookRunner` from config in `RecipeBuilder`**

In `src/recipe/builder.rs`, add an import at the top:

```rust
use super::hook::HookRunner;
use crate::config::PostBuildHook;
```

Add a helper method to `RecipeBuilder`:

```rust
fn build_hook(&self, cfg: Option<&PostBuildHook>, target_name: &str) -> Option<HookRunner> {
    cfg.map(|hook_cfg| {
        let mut vars = self.env.clone();
        vars.insert("TARGET".to_string(), delbin::Value::String(target_name.to_string()));
        HookRunner {
            command: hook_cfg.command.clone(),
            arg_templates: hook_cfg.args.clone(),
            vars,
        }
    })
}
```

Update `build_merge` — add `hook` to the returned `MergeRecipe`:

```rust
fn build_merge(&self, name: &str, t: &MergeTarget) -> Result<MergeRecipe, RecipeError> {
    // ... existing code unchanged ...
    Ok(MergeRecipe {
        name: name.to_string(),
        description: t.description.clone(),
        bootloader_reader,
        app_reader,
        writer,
        output_path,
        hook: self.build_hook(t.post_build.as_ref(), name),  // ADD THIS
    })
}
```

Update `build_convert`:

```rust
fn build_convert(&self, name: &str, t: &ConvertTarget) -> Result<ConvertRecipe, RecipeError> {
    // ... existing code unchanged ...
    Ok(ConvertRecipe {
        name: name.to_string(),
        description: t.description.clone(),
        reader,
        writer,
        output_path,
        hook: self.build_hook(t.post_build.as_ref(), name),  // ADD THIS
    })
}
```

Update `build_pack`:

```rust
fn build_pack(&self, name: &str, t: &PackTarget) -> Result<PackRecipe, RecipeError> {
    // ... existing code unchanged ...
    Ok(PackRecipe {
        name: name.to_string(),
        description: t.description.clone(),
        app_reader,
        writer,
        output_path,
        header_builder,
        hook: self.build_hook(t.post_build.as_ref(), name),  // ADD THIS
    })
}
```

- [ ] **Step 3: Run all tests**

```bash
cargo test
```

Expected: all existing tests pass; no regressions

- [ ] **Step 4: Manual smoke test** (optional, requires a real hex file)

Add a `[targets.test.post_build]` block to `xx.baker.toml`:

```toml
[targets.test_hook.post_build]
command = "cmd"                # Windows; use "sh" on Unix
args = ["/c", "echo", "Hook ran! Output: ${OUTPUT_FILE}"]
```

Run:
```bash
cargo run -- -c xx.baker.toml build test_hook
```

Expected: normal build output + `Hook ran! Output: <path>.hex` printed

- [ ] **Step 5: Commit**

```bash
git add src/recipe/builder.rs src/recipe/merge.rs src/recipe/convert.rs src/recipe/pack.rs
git commit -m "feat(recipe): wire post_build hook into all recipe types"
```

---

## Summary

After all 4 tasks, the full flow is:

```
baker.toml [post_build]
    → Config::PostBuildHook (deserialized)
    → RecipeBuilder::build_hook() → HookRunner { command, arg_templates, vars }
    → Recipe.cook() → writes output → HookRunner::run(output_path)
        → expands ${OUTPUT_FILE}, ${OUTPUT_DIR}, ${OUTPUT_NAME}, ${VER.*}, etc.
        → std::process::Command::new(command).args(expanded_args).output()
        → Ok(()) or ExternalToolNotFound / ExternalToolFailed
```

JFlash example in `baker.toml`:

```toml
[targets.production]
type = "merge"
bootloader = "bl"
app_file = "build/app.hex"
output_format = "hex"
output_name = "fw_${VER.MAJOR}.${VER.MINOR}"

[targets.production.post_build]
command = "jflash.exe"
args = [
    "-openprj", "config/STM32F4.jflash",
    "-open",    "${OUTPUT_FILE}",
    "-savecfg", "${OUTPUT_DIR}/${OUTPUT_NAME}.cfg",
    "-savedat", "${OUTPUT_DIR}/${OUTPUT_NAME}.dat",
    "-hide"
]
```
