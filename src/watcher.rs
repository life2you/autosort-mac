use std::path::PathBuf;
use std::sync::mpsc;

use anyhow::{Context, Result};
use notify::{
    Config as NotifyConfig, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher,
};

use crate::config::Config;
use crate::sorter::{sort_file, SortOutcome};

pub fn run(config: Config) -> Result<()> {
    let watch_dirs = config.expanded_watch_dirs()?;
    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();

    let mut watcher = RecommendedWatcher::new(
        move |result| {
            if tx.send(result).is_err() {
                eprintln!("文件事件通道已关闭");
            }
        },
        NotifyConfig::default(),
    )
    .context("无法创建文件监听器")?;

    println!("autosort-mac 已启动，正在监听：");
    for dir in &watch_dirs {
        if !dir.exists() {
            println!("忽略不存在的目录：{}", dir.display());
            continue;
        }
        watcher
            .watch(dir, RecursiveMode::NonRecursive)
            .with_context(|| format!("无法监听目录：{}", dir.display()))?;
        println!("  - {}", dir.display());
    }
    println!("按 Ctrl+C 可停止前台运行。");

    for result in rx {
        match result {
            Ok(event) => handle_event(event, &config),
            Err(error) => eprintln!("监听事件错误：{error}"),
        }
    }

    Ok(())
}

fn handle_event(event: Event, config: &Config) {
    if !is_relevant_event(&event.kind) {
        return;
    }

    for path in event.paths {
        if path.is_file() {
            process_path(path, config);
        }
    }
}

fn process_path(path: PathBuf, config: &Config) {
    match sort_file(&path, config) {
        Ok(SortOutcome::Moved { from, to }) => {
            println!("已移动：{} -> {}", from.display(), to.display());
        }
        Ok(SortOutcome::Ignored { path, reason }) => {
            println!("已忽略：{} ({reason})", path.display());
        }
        Err(error) => {
            eprintln!("处理失败：{}：{error:#}", path.display());
        }
    }
}

fn is_relevant_event(kind: &EventKind) -> bool {
    matches!(
        kind,
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Any
    )
}
