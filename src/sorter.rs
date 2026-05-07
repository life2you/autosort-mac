use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Local;

use crate::config::{Config, SortMode};

pub enum SortOutcome {
    Moved { from: PathBuf, to: PathBuf },
    Ignored { path: PathBuf, reason: String },
}

pub fn sort_file(path: &Path, config: &Config) -> Result<SortOutcome> {
    if should_ignore_path(path, config)? {
        return Ok(SortOutcome::Ignored {
            path: path.to_path_buf(),
            reason: "隐藏文件、非普通文件或临时下载文件".to_string(),
        });
    }

    wait_until_ready(path, config)?;

    if should_ignore_path(path, config)? {
        return Ok(SortOutcome::Ignored {
            path: path.to_path_buf(),
            reason: "等待后文件已不可处理或应被忽略".to_string(),
        });
    }

    let target_dir = build_target_dir(path, config)?;
    fs::create_dir_all(&target_dir)
        .with_context(|| format!("无法创建目标目录：{}", target_dir.display()))?;

    let file_name = path
        .file_name()
        .context("无法读取文件名")?
        .to_string_lossy()
        .to_string();
    let destination = unique_destination(&target_dir, &file_name);

    fs::rename(path, &destination).or_else(|_| {
        fs::copy(path, &destination)
            .with_context(|| {
                format!(
                    "无法复制文件：{} -> {}",
                    path.display(),
                    destination.display()
                )
            })
            .and_then(|_| {
                fs::remove_file(path).with_context(|| format!("无法删除原文件：{}", path.display()))
            })
    })?;

    Ok(SortOutcome::Moved {
        from: path.to_path_buf(),
        to: destination,
    })
}

fn wait_until_ready(path: &Path, config: &Config) -> Result<()> {
    loop {
        wait_until_stable(path, config.wait_seconds)?;

        if !config.skip_open_files || !is_file_open(path)? {
            return Ok(());
        }

        println!("文件当前仍被其他应用打开，继续等待：{}", path.display());
        thread::sleep(Duration::from_secs(config.wait_seconds.max(1)));
    }
}

fn should_ignore_path(path: &Path, config: &Config) -> Result<bool> {
    let file_name = match path.file_name().and_then(|name| name.to_str()) {
        Some(name) => name,
        None => return Ok(true),
    };

    if file_name.starts_with('.') {
        return Ok(true);
    }

    let metadata = match fs::metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(error) => {
            return Err(error).with_context(|| format!("无法读取文件信息：{}", path.display()))
        }
    };

    if !metadata.is_file() {
        return Ok(true);
    }

    let extension = extension_for(path);
    Ok(config
        .ignore_extensions
        .iter()
        .any(|ignored| ignored.eq_ignore_ascii_case(&extension)))
}

fn wait_until_stable(path: &Path, wait_seconds: u64) -> Result<()> {
    let delay = Duration::from_secs(wait_seconds.max(1));
    let mut previous_size = file_size(path)?;

    loop {
        thread::sleep(delay);
        let current_size = file_size(path)?;
        if current_size == previous_size {
            return Ok(());
        }
        previous_size = current_size;
        println!(
            "文件仍在写入，继续等待：{} ({} bytes)",
            path.display(),
            current_size
        );
    }
}

fn file_size(path: &Path) -> Result<u64> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.len()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(error).with_context(|| format!("无法读取文件大小：{}", path.display())),
    }
}

fn is_file_open(path: &Path) -> Result<bool> {
    let output = Command::new("lsof")
        .arg("--")
        .arg(path)
        .output()
        .with_context(|| format!("无法检查文件是否被打开：{}", path.display()))?;

    Ok(output.status.success() && !output.stdout.is_empty())
}

fn build_target_dir(path: &Path, config: &Config) -> Result<PathBuf> {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let extension = extension_for(path);
    let target_root = config.expanded_target_dir()?;

    Ok(match config.mode()? {
        SortMode::DateThenExt => target_root.join(date).join(extension),
        SortMode::ExtThenDate => target_root.join(extension).join(date),
    })
}

fn extension_for(path: &Path) -> String {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "no_extension".to_string())
}

fn unique_destination(target_dir: &Path, file_name: &str) -> PathBuf {
    let candidate = target_dir.join(file_name);
    if !candidate.exists() {
        return candidate;
    }

    let timestamp = Local::now().format("%H%M%S").to_string();
    let source = Path::new(file_name);
    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(file_name);

    match source.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if !ext.is_empty() => target_dir.join(format!("{stem}_{timestamp}.{ext}")),
        _ => target_dir.join(format!("{stem}_{timestamp}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_falls_back_to_no_extension() {
        assert_eq!(extension_for(Path::new("README")), "no_extension");
    }

    #[test]
    fn extension_is_lowercase() {
        assert_eq!(extension_for(Path::new("demo.PDF")), "pdf");
    }
}
