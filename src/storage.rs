//! Runtime storage selected by a marker beside the executable.
use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub struct StoragePaths {
    pub config: PathBuf,
    pub data: PathBuf,
    pub logs: PathBuf,
    pub temporary: PathBuf,
}

impl StoragePaths {
    fn for_executable(executable: &Path) -> Result<Self, String> {
        let directory = executable
            .parent()
            .ok_or("Executable directory is unavailable")?;
        if directory.join("portable.txt").is_file() {
            return Ok(Self {
                config: directory.to_owned(),
                data: directory.join("data"),
                logs: directory.join("logs"),
                temporary: directory.join("tmp"),
            });
        }
        let dirs = directories::ProjectDirs::from("org", "PicoForge", "PicoForge All")
            .ok_or("Software settings directory is unavailable")?;
        Ok(Self {
            config: dirs.config_dir().into(),
            data: dirs.data_local_dir().into(),
            logs: dirs.data_local_dir().join("logs"),
            temporary: std::env::temp_dir(),
        })
    }
}

static PATHS: LazyLock<Result<StoragePaths, String>> = LazyLock::new(|| {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    StoragePaths::for_executable(&executable)
});

pub fn paths() -> Result<&'static StoragePaths, String> {
    PATHS.as_ref().map_err(Clone::clone)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn marker_contents_do_not_matter_and_all_storage_stays_beside_the_executable() {
        let root =
            std::env::temp_dir().join(format!("picoforge-portable-{}", rand::random::<u64>()));
        std::fs::create_dir(&root).unwrap();
        let executable = root.join("picoforge.exe");
        let standard = StoragePaths::for_executable(&executable).unwrap();
        assert!(!standard.config.starts_with(&root));
        assert!(!standard.data.starts_with(&root));
        for contents in [b"".as_slice(), b"anything", &[0xff, 0, 0xfe]] {
            std::fs::write(root.join("portable.txt"), contents).unwrap();
            let portable = StoragePaths::for_executable(&executable).unwrap();
            assert_eq!(portable.config, root);
            assert_eq!(portable.data, root.join("data"));
            assert_eq!(portable.logs, root.join("logs"));
            assert_eq!(portable.temporary, root.join("tmp"));
        }
        std::fs::remove_file(root.join("portable.txt")).unwrap();
        let standard_again = StoragePaths::for_executable(&executable).unwrap();
        assert_eq!(standard.config, standard_again.config);
        assert_eq!(standard.data, standard_again.data);
        std::fs::remove_dir(root).unwrap();
    }
}
