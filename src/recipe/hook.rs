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
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Err(RecipeError::ExternalToolFailed {
                command: self.command.clone(),
                exit_code,
                stdout,
                stderr,
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        // ${OUTPUT_FILE} is injected by run() — should not produce MissingVariable
        #[cfg(windows)]
        let runner = make_runner("cmd", &["/c", "echo", "${OUTPUT_FILE}"], &[]);
        #[cfg(not(windows))]
        let runner = make_runner("sh", &["-c", "echo ${OUTPUT_FILE}"], &[]);

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
