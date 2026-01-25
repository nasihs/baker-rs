use std::path::Path;

use super::error::ConfigError;
use super::schema::{Config, Group, Target, Bootloader};

#[allow(dead_code)]
fn validate(config: &Config) -> Result<(), ConfigError> {  // TODO move to impl 
    // 验证 targets 中引用的 bootloader 存在
    for (_name, target) in &config.targets {
        if let Target::Merge(merge) = target {
            // 检查是否是 bootloader 引用（非路径）
            if !merge.bootloader.contains('/') && !merge.bootloader.contains('\\') {
                if !config.bootloaders.contains_key(&merge.bootloader) {
                    // 也可能是直接路径，暂时跳过严格检查
                    // 或者检查文件是否存在
                }
            }
        }
    }

    for (group_name, group) in &config.groups {
        for target_name in group.targets() {
            if !config.targets.contains_key(target_name) {
                return Err(ConfigError::InvalidTarget {
                    name: target_name.clone(),
                    reason: format!("referenced in group '{}' but not defined", group_name),
                });
            }
        }
    }

    if !config.targets.contains_key(&config.project.default) && !config.groups.contains_key(&config.project.default) {
        return Err(ConfigError::InvalidTarget {
            name: config.project.default.clone(),
            reason: "specified as default but not defined as target or group".to_string(),
        });
    }

    Ok(())
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        if !path.exists() {
            return Err(ConfigError::NotFound { path: path.to_path_buf() });
        }
        let content = std::fs::read_to_string(path)?;

        Self::from_str(&content)
    }

    pub fn from_str(content: &str) -> Result<Self, ConfigError> {
        let config = toml::from_str(content)?;

        Self::validate(&config)?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        


        Ok(())
    }


    #[allow(unused_variables, dead_code)]
    fn validate_bootloader(&self, bl: &Bootloader) -> Result<(), ConfigError> {
        todo!()
    }

    // fn validate_header(&self, header: &Hea)
    #[allow(unused_variables, dead_code)]
    fn validate_target(&self, target: &Target) -> Result<(), ConfigError> {
        todo!()
    }

    #[allow(unused_variables, dead_code)]
    fn validate_group(&self, group: &Group) -> Result<(), ConfigError> {
        todo!()
    }

    // resolve the targets to be build
    // if targets are specified, build the specified
    // or build the default
    pub fn resolve_targets<'a>(&'a self, specified: &'a [String]) -> Result<Vec<&'a str>, ConfigError> {
        let mut result = Vec::new();

        if !specified.is_empty() {
            for name in specified {
                self.expand_target_or_group(name, &mut result)?;
            }
        } else {
            self.expand_target_or_group(&self.project.default, &mut result)?;
        }

        return Ok(result);
    }

    fn expand_target_or_group<'a>(
        &'a self,
        name: &'a str,
        result: &mut Vec<&'a str>,
    ) -> Result<(), ConfigError> {
        if self.targets.contains_key(name) {
            if !result.contains(&name) {
                result.push(name);
            }
            return Ok(());
        }

        if let Some(group) = self.groups.get(name) {
            for target_name in group.targets() {
                if self.targets.contains_key(target_name) {
                    if !result.contains(&target_name.as_str()) {
                        result.push(target_name);
                    }
                } else {
                    return Err(ConfigError::TargetUndefined(target_name.to_owned()));
                }
            }
            return Ok(());
        }

        Err(ConfigError::TargetUndefined(name.to_string()))
    }

}
