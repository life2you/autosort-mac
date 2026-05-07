use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{bail, Context, Result};

const LABEL: &str = "com.user.mac-file-auto-sort";
const PLIST_NAME: &str = "com.user.mac-file-auto-sort.plist";

#[derive(Debug)]
pub struct LaunchdStatus {
    pub installed: bool,
    pub running: bool,
}

pub fn install() -> Result<()> {
    let plist_path = plist_path()?;
    if plist_path.exists() {
        println!("launchd 服务已安装：{}", plist_path.display());
        return Ok(());
    }

    let binary = std::env::current_exe().context("无法定位当前 autosort-mac 二进制")?;
    let parent = plist_path.parent().context("无法定位 LaunchAgents 目录")?;
    fs::create_dir_all(parent).with_context(|| format!("无法创建目录：{}", parent.display()))?;

    fs::write(&plist_path, plist_content(&binary))
        .with_context(|| format!("无法写入 plist：{}", plist_path.display()))?;

    load_service()?;
    println!("launchd 服务已安装并启动：{}", plist_path.display());
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let plist_path = plist_path()?;
    if !plist_path.exists() {
        println!("launchd 服务未安装。");
        return Ok(());
    }

    let _ = unload_service();
    fs::remove_file(&plist_path)
        .with_context(|| format!("无法删除 plist：{}", plist_path.display()))?;
    println!("launchd 服务已卸载。");
    Ok(())
}

pub fn stop() -> Result<()> {
    if !plist_path()?.exists() {
        println!("launchd 服务未安装。");
        return Ok(());
    }

    unload_service()?;
    println!("launchd 服务已临时停止，plist 文件仍保留。");
    Ok(())
}

pub fn restart() -> Result<()> {
    if !plist_path()?.exists() {
        println!("launchd 服务未安装，请先执行 autosort-mac install。");
        return Ok(());
    }

    let _ = unload_service();
    load_service()?;
    println!("launchd 服务已重启。");
    Ok(())
}

pub fn status() -> Result<LaunchdStatus> {
    let installed = plist_path()?.exists();
    let running = if installed { is_running()? } else { false };
    Ok(LaunchdStatus { installed, running })
}

pub fn plist_path() -> Result<PathBuf> {
    let home = dirs::home_dir().context("无法定位用户 Home 目录")?;
    Ok(home.join("Library").join("LaunchAgents").join(PLIST_NAME))
}

fn plist_content(binary: &std::path::Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>{LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>{}</string>
    <string>start</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardOutPath</key>
  <string>/tmp/mac-file-auto-sort.log</string>
  <key>StandardErrorPath</key>
  <string>/tmp/mac-file-auto-sort.error.log</string>
</dict>
</plist>
"#,
        escape_plist_string(&binary.display().to_string())
    )
}

fn load_service() -> Result<()> {
    let path = plist_path()?;
    let gui_target = gui_target()?;

    let output = Command::new("launchctl")
        .args(["bootstrap", &gui_target])
        .arg(&path)
        .output()
        .context("无法执行 launchctl bootstrap")?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("Bootstrap failed: 5") || stderr.contains("service already loaded") {
        return Ok(());
    }

    bail!("启动 launchd 服务失败：{}", stderr.trim());
}

fn unload_service() -> Result<()> {
    let output = Command::new("launchctl")
        .args(["bootout", &service_target()?])
        .output()
        .context("无法执行 launchctl bootout")?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("No such process")
        || stderr.contains("Could not find service")
        || stderr.contains("No such file or directory")
    {
        return Ok(());
    }

    bail!("停止 launchd 服务失败：{}", stderr.trim());
}

fn is_running() -> Result<bool> {
    let output = Command::new("launchctl")
        .args(["print", &service_target()?])
        .output()
        .context("无法执行 launchctl print")?;
    Ok(output.status.success())
}

fn gui_target() -> Result<String> {
    Ok(format!("gui/{}", unsafe { libc_getuid() }))
}

fn service_target() -> Result<String> {
    Ok(format!("{}/{}", gui_target()?, LABEL))
}

#[cfg(unix)]
unsafe fn libc_getuid() -> u32 {
    extern "C" {
        fn getuid() -> u32;
    }
    getuid()
}

fn escape_plist_string(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
