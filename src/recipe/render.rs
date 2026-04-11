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
