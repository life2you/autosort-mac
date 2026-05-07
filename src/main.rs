mod config;
mod launchd;
mod sorter;
mod watcher;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::config::{config_exists, config_path, Config};

#[derive(Parser)]
#[command(name = "autosort-mac")]
#[command(
    version,
    about = "Automatically organize Desktop and Downloads files on macOS"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create the default config file.
    Init,
    /// Start watching files in the foreground.
    Start,
    /// Print current config and launchd status.
    Status,
    /// Install and start the launchd background service.
    Install,
    /// Stop and remove the launchd background service.
    Uninstall,
    /// Temporarily stop the launchd service but keep the plist.
    Stop,
    /// Restart the launchd service.
    Restart,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            Config::create_default_if_missing()?;
        }
        Commands::Start => {
            ensure_config_exists()?;
            let config = Config::load()?;
            watcher::run(config)?;
        }
        Commands::Status => {
            print_status()?;
        }
        Commands::Install => {
            ensure_config_exists()?;
            launchd::install()?;
        }
        Commands::Uninstall => {
            launchd::uninstall()?;
        }
        Commands::Stop => {
            launchd::stop()?;
        }
        Commands::Restart => {
            launchd::restart()?;
        }
    }

    Ok(())
}

fn ensure_config_exists() -> Result<()> {
    if !config_exists()? {
        println!(
            "配置文件不存在：{}\n请先执行：autosort-mac init",
            config_path()?.display()
        );
        std::process::exit(1);
    }
    Ok(())
}

fn print_status() -> Result<()> {
    let path = config_path()?;
    println!("配置文件路径：{}", path.display());

    if !path.exists() {
        println!("配置文件不存在，请先执行：autosort-mac init");
    } else {
        let config = Config::load()?;
        println!("监控目录：");
        for dir in config.expanded_watch_dirs()? {
            println!("  - {}", dir.display());
        }
        println!("目标目录：{}", config.expanded_target_dir()?.display());
        println!("整理模式：{}", config.mode);
        println!("忽略的后缀：{}", config.ignore_extensions.join(", "));
        println!("wait_seconds：{}", config.wait_seconds);
        println!("跳过打开中的文件：{}", yes_no(config.skip_open_files));
    }

    let launchd_status = launchd::status()?;
    println!(
        "launchd 服务是否已安装：{}",
        yes_no(launchd_status.installed)
    );
    println!(
        "launchd 服务是否正在运行：{}",
        yes_no(launchd_status.running)
    );

    Ok(())
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "是"
    } else {
        "否"
    }
}
