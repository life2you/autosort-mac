use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

pub const APP_DIR_NAME: &str = "mac-file-auto-sort";
pub const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub watch_dirs: Vec<String>,
    pub target_dir: String,
    pub mode: String,
    pub ignore_extensions: Vec<String>,
    pub wait_seconds: u64,
    #[serde(default = "default_skip_open_files")]
    pub skip_open_files: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortMode {
    DateThenExt,
    ExtThenDate,
}

impl SortMode {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "date_then_ext" => Ok(Self::DateThenExt),
            "ext_then_date" => Ok(Self::ExtThenDate),
            other => bail!("配置中的 mode 无效：{other}。可用值：date_then_ext 或 ext_then_date"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watch_dirs: vec!["~/Desktop".to_string(), "~/Downloads".to_string()],
            target_dir: "~/FileAutoSort".to_string(),
            mode: "date_then_ext".to_string(),
            ignore_extensions: vec![
                "crdownload".to_string(),
                "download".to_string(),
                "part".to_string(),
                "tmp".to_string(),
            ],
            wait_seconds: 3,
            skip_open_files: true,
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("无法读取配置文件：{}", path.display()))?;
        let config: Self = toml::from_str(&content)
            .with_context(|| format!("配置文件格式不正确：{}", path.display()))?;
        config.validate()?;
        Ok(config)
    }

    pub fn create_default_if_missing() -> Result<PathBuf> {
        let path = config_path()?;
        if path.exists() {
            println!("配置文件已存在，不会覆盖：{}", path.display());
            return Ok(path);
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建配置目录：{}", parent.display()))?;
        }

        let content = toml::to_string_pretty(&Self::default()).context("无法生成默认配置")?;
        fs::write(&path, content)
            .with_context(|| format!("无法写入配置文件：{}", path.display()))?;
        println!("已创建默认配置：{}", path.display());
        Ok(path)
    }

    pub fn validate(&self) -> Result<()> {
        if self.watch_dirs.is_empty() {
            bail!("配置中的 watch_dirs 不能为空");
        }
        SortMode::parse(&self.mode)?;
        Ok(())
    }

    pub fn mode(&self) -> Result<SortMode> {
        SortMode::parse(&self.mode)
    }

    pub fn expanded_watch_dirs(&self) -> Result<Vec<PathBuf>> {
        self.watch_dirs
            .iter()
            .map(|path| expand_tilde(path))
            .collect()
    }

    pub fn expanded_target_dir(&self) -> Result<PathBuf> {
        expand_tilde(&self.target_dir)
    }
}

pub fn config_path() -> Result<PathBuf> {
    let config_root = dirs::config_dir().context("无法定位用户配置目录")?;
    Ok(config_root.join(APP_DIR_NAME).join(CONFIG_FILE_NAME))
}

pub fn expand_tilde(path: &str) -> Result<PathBuf> {
    if path == "~" {
        return dirs::home_dir().context("无法定位用户 Home 目录");
    }

    if let Some(stripped) = path.strip_prefix("~/") {
        let home = dirs::home_dir().context("无法定位用户 Home 目录")?;
        return Ok(home.join(stripped));
    }

    Ok(Path::new(path).to_path_buf())
}

pub fn config_exists() -> Result<bool> {
    Ok(config_path()?.exists())
}

fn default_skip_open_files() -> bool {
    true
}
