use std::path::Path;

use super::error::ConfigError;
use super::schema::{Config, Group, Target};

/// 从文件加载配置
pub fn load(path: &Path) -> Result<Config, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::NotFound {
            path: path.to_path_buf(),
        });
    }

    let content = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&content)?;

    validate(&config)?;

    Ok(config)
}

/// 验证配置有效性
fn validate(config: &Config) -> Result<(), ConfigError> {
    // 验证 targets 中引用的 bootloader 存在
    for (name, target) in &config.targets {
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

    // 验证 groups 中引用的 targets 存在
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

    // 验证 default 引用存在
    if let Some(default) = &config.project.default {
        if !config.targets.contains_key(default) && !config.groups.contains_key(default) {
            return Err(ConfigError::InvalidTarget {
                name: default.clone(),
                reason: "specified as default but not defined as target or group".to_string(),
            });
        }
    }

    Ok(())
}

impl Config {
    /// 解析要构建的 targets
    /// 如果指定了 targets，使用指定的
    /// 否则使用 default
    pub fn resolve_targets<'a>(&'a self, specified: &'a [String]) -> Result<Vec<&'a str>, ConfigError> {
        if !specified.is_empty() {
            // 展开所有指定的 targets 和 groups
            let mut result = Vec::new();
            for name in specified {
                self.expand_target_or_group(name, &mut result)?;
            }
            return Ok(result);
        }

        // 使用 default
        if let Some(default) = &self.project.default {
            let mut result = Vec::new();
            self.expand_target_or_group(default, &mut result)?;
            return Ok(result);
        }

        // 无 default 且未指定，构建所有 targets
        Ok(self.targets.keys().map(|s| s.as_str()).collect())
    }

    /// 展开 target 或 group
    fn expand_target_or_group<'a>(
        &'a self,
        name: &'a str,
        result: &mut Vec<&'a str>,
    ) -> Result<(), ConfigError> {
        // 先检查是否是 target
        if self.targets.contains_key(name) {
            if !result.contains(&name) {
                result.push(name);
            }
            return Ok(());
        }

        // 再检查是否是 group
        if let Some(group) = self.groups.get(name) {
            for target_name in group.targets() {
                if self.targets.contains_key(target_name) {
                    if !result.contains(&target_name.as_str()) {
                        result.push(target_name);
                    }
                } else {
                    return Err(ConfigError::TargetNotFound(target_name.to_owned()));
                }
            }
            return Ok(());
        }

        Err(ConfigError::TargetNotFound(name.to_string()))
    }

    /// 获取 bootloader 路径
    pub fn get_bootloader_path(&self, reference: &str) -> Option<&Path> {
        // 先查找 bootloaders 定义
        if let Some(bl) = self.bootloaders.get(reference) {
            return Some(&bl.file);
        }
        // 否则当作直接路径
        None
    }
}
