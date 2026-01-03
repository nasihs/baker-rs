use std::path::{Path, PathBuf};

pub struct BuildContext {
    pub base_dir: PathBuf,
    pub output_dir: PathBuf,
}

impl BuildContext {
    pub fn new(config_path: &Path, output_dir: &Path) -> Self {
        let base_dir = config_path
            .parent()
            .unwrap_or(Path::new("."))
            .to_path_buf();

        Self {
            base_dir,
            output_dir: output_dir.to_path_buf(),
        }
    }

    // convert relative path to absolute path
    pub fn resolve_path(&self, path: &Path) -> PathBuf {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.base_dir.join(path)
        }
    }

    pub fn output_path(&self, filename: &str) -> PathBuf {
        self.output_dir.join(filename)
    }
}


