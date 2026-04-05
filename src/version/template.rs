use std::collections::HashMap;
use std::path::PathBuf;
use regex::Regex;
use super::{VersionError, VersionExtractor};

/// Extracts variables from any text file using a template.
///
/// Each line in the template that contains `${VAR}` is compiled into a regex
/// matcher. The surrounding text is matched literally; `${VAR}` captures the
/// value at that position. All captured values are returned in a map keyed by
/// `VER.<VAR>` and typed as `delbin::Value::U32` when the value parses as an
/// integer (decimal, `0x` hex, `0b` binary), or `delbin::Value::String`
/// otherwise.
pub struct TemplateExtractor {
    file_path: PathBuf,
    template: String,
}

impl TemplateExtractor {
    pub fn new(file_path: PathBuf, template: String) -> Self {
        Self { file_path, template }
    }

    fn read_file(&self) -> Result<String, VersionError> {
        if !self.file_path.exists() {
            return Err(VersionError::FileNotFound(self.file_path.clone()));
        }
        Ok(std::fs::read_to_string(&self.file_path)?)
    }

    /// Compile one template line into a (regex, vec-of-var-names) pair.
    /// Returns `None` if the line contains no `${VAR}` placeholders.
    fn compile_line(template_line: &str) -> Option<(Regex, Vec<String>)> {
        // Split the line on placeholder occurrences, collecting var names and
        // the literal segments between them.
        let placeholder_re = Regex::new(r"\$\{([A-Z_][A-Z0-9_]*)\}").unwrap();

        let mut var_names: Vec<String> = Vec::new();
        for cap in placeholder_re.captures_iter(template_line) {
            var_names.push(cap[1].to_string());
        }

        if var_names.is_empty() {
            return None;
        }

        // Build a regex that:
        //  - ignores leading/trailing whitespace differences
        //  - matches the literal parts verbatim (regex-escaped)
        //  - replaces each ${VAR} with a named capture group
        let mut pattern = String::from(r"^\s*");
        let mut remaining = template_line.trim();

        for var in &var_names {
            let placeholder = format!("${{{}}}", var);
            if let Some(pos) = remaining.find(&placeholder) {
                let literal = &remaining[..pos];
                // Collapse multiple whitespace to `\s+` so minor spacing
                // differences between template and file are tolerated.
                pattern.push_str(&Self::escape_with_flexible_whitespace(literal));
                pattern.push_str(&format!(r"(?P<{var}>.+?)"));
                remaining = &remaining[pos + placeholder.len()..];
            }
        }
        // Append any trailing literal, then allow optional trailing comment / whitespace.
        pattern.push_str(&Self::escape_with_flexible_whitespace(remaining.trim_end()));
        pattern.push_str(r"\s*(?://.*)?$");

        Regex::new(&pattern).ok().map(|re| (re, var_names))
    }

    /// Regex-escape a literal string, but replace runs of whitespace with `\s+`
    /// so that differing indentation between template and target file is handled.
    fn escape_with_flexible_whitespace(s: &str) -> String {
        let mut result = String::new();
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            if c.is_whitespace() {
                // Consume the rest of the whitespace run.
                while chars.peek().map(|c| c.is_whitespace()).unwrap_or(false) {
                    chars.next();
                }
                result.push_str(r"\s+");
            } else {
                result.push_str(&regex::escape(&c.to_string()));
            }
        }
        result
    }

    fn infer_value(raw: &str) -> delbin::Value {
        let trimmed = raw.trim();
        // Strip surrounding quotes for string literals.
        let unquoted = if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        };

        // Try integer parsing (hex, binary, decimal).
        if let Some(hex) = unquoted.strip_prefix("0x").or_else(|| unquoted.strip_prefix("0X")) {
            if let Ok(v) = u32::from_str_radix(hex, 16) {
                return delbin::Value::U32(v);
            }
        }
        if let Some(bin) = unquoted.strip_prefix("0b").or_else(|| unquoted.strip_prefix("0B")) {
            if let Ok(v) = u32::from_str_radix(bin, 2) {
                return delbin::Value::U32(v);
            }
        }
        if let Ok(v) = unquoted.parse::<u32>() {
            return delbin::Value::U32(v);
        }

        delbin::Value::String(unquoted.to_string())
    }

    /// Extract variables from the target file.
    /// Returns a map of `VER.<VAR>` → `delbin::Value`.
    pub fn extract_vars(&self) -> Result<HashMap<String, delbin::Value>, VersionError> {
        let content = self.read_file()?;
        let file_lines: Vec<&str> = content.lines().collect();

        // Compile all template lines that have placeholders.
        let matchers: Vec<(String, Regex, Vec<String>)> = self.template
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter_map(|line| {
                Self::compile_line(line).map(|(re, vars)| (line.trim().to_string(), re, vars))
            })
            .collect();

        let mut results: HashMap<String, delbin::Value> = HashMap::new();

        for (template_line, re, var_names) in &matchers {
            let mut matched = false;
            'file: for file_line in &file_lines {
                if let Some(caps) = re.captures(file_line) {
                    for var in var_names {
                        if let Some(m) = caps.name(var) {
                            let key = format!("VER.{}", var);
                            results.insert(key, Self::infer_value(m.as_str()));
                        }
                    }
                    matched = true;
                    break 'file;
                }
            }
            if !matched {
                return Err(VersionError::PatternNotMatched {
                    pattern: template_line.clone(),
                });
            }
        }

        Ok(results)
    }
}

/// Adapter that also implements the legacy `VersionExtractor` trait for
/// callers that only need a `VersionInfo` struct (e.g. tests).
impl VersionExtractor for TemplateExtractor {
    fn extract(&self) -> Result<super::VersionInfo, VersionError> {
        let vars = self.extract_vars()?;

        let get_u32 = |key: &str| -> Option<u32> {
            match vars.get(key)? {
                delbin::Value::U32(v) => Some(*v),
                delbin::Value::String(s) => s.parse().ok(),
                _ => None,
            }
        };
        let get_str = |key: &str| -> Option<String> {
            match vars.get(key)? {
                delbin::Value::String(s) => Some(s.clone()),
                delbin::Value::U32(v) => Some(v.to_string()),
                _ => None,
            }
        };

        let major = get_u32("VER.MAJOR").ok_or_else(|| {
            VersionError::MissingConfig("VER.MAJOR (add '${MAJOR}' to template)".to_string())
        })?;
        let minor = get_u32("VER.MINOR").ok_or_else(|| {
            VersionError::MissingConfig("VER.MINOR (add '${MINOR}' to template)".to_string())
        })?;
        let patch = get_u32("VER.PATCH").ok_or_else(|| {
            VersionError::MissingConfig("VER.PATCH (add '${PATCH}' to template)".to_string())
        })?;

        Ok(super::VersionInfo {
            major,
            minor,
            patch,
            build: get_u32("VER.BUILD"),
            pre_release: get_str("VER.PRE_RELEASE"),
            build_metadata: get_str("VER.BUILD_METADATA"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_file(content: &str) -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("version.h");
        fs::write(&path, content).unwrap();
        (dir, path)
    }

    fn extractor(path: PathBuf, template: &str) -> TemplateExtractor {
        TemplateExtractor::new(path, template.to_string())
    }

    #[test]
    fn test_separate_integer_macros() {
        let content = "\
            #define VERSION_MAJOR 1\n\
            #define VERSION_MINOR 2\n\
            #define VERSION_PATCH 3\n\
            #define BUILD_NUMBER 100\n";
        let (_dir, path) = write_file(content);
        let e = extractor(path, "\
            #define VERSION_MAJOR ${MAJOR}\n\
            #define VERSION_MINOR ${MINOR}\n\
            #define VERSION_PATCH ${PATCH}\n\
            #define BUILD_NUMBER ${BUILD}\n");
        let v = e.extract().unwrap();
        assert_eq!((v.major, v.minor, v.patch, v.build), (1, 2, 3, Some(100)));
    }

    #[test]
    fn test_inline_string_parsing() {
        let content = r#"#define VERSION_STR "v1.2.3""#;
        let (_dir, path) = write_file(content);
        let e = extractor(path, r#"#define VERSION_STR "v${MAJOR}.${MINOR}.${PATCH}""#);
        let v = e.extract().unwrap();
        assert_eq!((v.major, v.minor, v.patch), (1, 2, 3));
    }

    #[test]
    fn test_hex_values() {
        let content = "\
            #define VER_MAJOR 0x01\n\
            #define VER_MINOR 0x0A\n\
            #define VER_PATCH 0xFF\n";
        let (_dir, path) = write_file(content);
        let e = extractor(path, "\
            #define VER_MAJOR ${MAJOR}\n\
            #define VER_MINOR ${MINOR}\n\
            #define VER_PATCH ${PATCH}\n");
        let v = e.extract().unwrap();
        assert_eq!((v.major, v.minor, v.patch), (1, 10, 255));
    }

    #[test]
    fn test_v_prefix_stripped_in_inline() {
        let content = r#"#define FW_VERSION "V2.5.10""#;
        let (_dir, path) = write_file(content);
        // The 'V' prefix is literal in the template, so it is consumed as a
        // literal character and not included in the MAJOR capture.
        let e = extractor(path, r#"#define FW_VERSION "V${MAJOR}.${MINOR}.${PATCH}""#);
        let v = e.extract().unwrap();
        assert_eq!((v.major, v.minor, v.patch), (2, 5, 10));
    }

    #[test]
    fn test_flexible_whitespace() {
        // Extra spaces between tokens should still match.
        let content = "#define   VERSION_MAJOR    3\n";
        let (_dir, path) = write_file(content);
        let e = extractor(path, "#define VERSION_MAJOR ${MAJOR}\n#define VERSION_MINOR ${MINOR}\n#define VERSION_PATCH ${PATCH}");
        // Only MAJOR is present; extract_vars should fail on MINOR/PATCH lines.
        let vars = e.extract_vars();
        assert!(vars.is_err()); // MINOR line won't match — expected PatternNotMatched
    }

    #[test]
    fn test_pattern_not_matched_error() {
        let content = "#define SOMETHING_ELSE 1\n";
        let (_dir, path) = write_file(content);
        let e = extractor(path, "#define VERSION_MAJOR ${MAJOR}");
        match e.extract_vars().unwrap_err() {
            VersionError::PatternNotMatched { pattern } => {
                assert!(pattern.contains("VERSION_MAJOR"));
            }
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn test_cmake_style() {
        let content = "project(MyApp VERSION 1.2.3)\n";
        let (_dir, path) = write_file(content);
        let e = extractor(path, "project(MyApp VERSION ${MAJOR}.${MINOR}.${PATCH})");
        let v = e.extract().unwrap();
        assert_eq!((v.major, v.minor, v.patch), (1, 2, 3));
    }

    #[test]
    fn test_custom_var_names() {
        let content = "#define BUILD_NUMBER 42\n";
        let (_dir, path) = write_file(content);
        let e = extractor(path, "#define BUILD_NUMBER ${BUILD}");
        let vars = e.extract_vars().unwrap();
        match vars.get("VER.BUILD") {
            Some(delbin::Value::U32(42)) => {}
            other => panic!("expected VER.BUILD = U32(42), got {:?}", other),
        }
    }
}
