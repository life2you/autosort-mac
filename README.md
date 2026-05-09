# mac-file-auto-sort

`mac-file-auto-sort` 是一个 macOS 文件自动整理工具，命令名为 `autosort-mac`。它可以监听 `~/Desktop` 和 `~/Downloads`，在新文件出现或下载完成后，按日期和文件后缀自动移动到配置好的目标目录。

## 功能特性

- 自动监听 Desktop 和 Downloads
- 按日期和文件后缀整理
- 避免移动下载中的临时文件
- 文件稳定后再移动
- 文件被其他应用打开时会继续等待
- 支持 launchd 后台常驻
- 支持 Homebrew 安装

## 安装方式

本地安装：

```bash
cargo install --path .
```

Homebrew tap 安装：

```bash
brew tap life2you/tap
brew install autosort-mac
```

本地 formula 安装：

```bash
brew install ./homebrew/autosort-mac.rb
```

## 使用方式

初始化配置：

```bash
autosort-mac init
```

前台运行：

```bash
autosort-mac start
```

查看状态：

```bash
autosort-mac status
```

安装后台服务：

```bash
autosort-mac install
```

停止后台服务：

```bash
autosort-mac stop
```

重启后台服务：

```bash
autosort-mac restart
```

卸载后台服务：

```bash
autosort-mac uninstall
```

## 配置说明

配置文件路径：

```text
~/.config/mac-file-auto-sort/config.toml
```

默认配置：

```toml
watch_dirs = ["~/Desktop", "~/Downloads"]
target_dir = "~/FileAutoSort"
mode = "date_then_ext"
ignore_extensions = ["crdownload", "download", "part", "tmp"]
wait_seconds = 3
skip_open_files = true
```

字段说明：

- `watch_dirs`：需要监听的目录列表，默认监听桌面和下载目录。
- `target_dir`：整理后的目标根目录。
- `mode`：整理模式，支持 `date_then_ext` 和 `ext_then_date`。
- `ignore_extensions`：需要忽略的临时文件后缀。
- `wait_seconds`：发现文件后等待的秒数；如果文件大小仍在变化，会继续等待。
- `skip_open_files`：为 `true` 时，如果文件仍被其他应用打开，会继续等待关闭后再移动。

## 目录结构示例

`date_then_ext`：

```text
~/FileAutoSort/2026-05-07/pdf/demo.pdf
~/FileAutoSort/2026-05-07/png/image.png
~/FileAutoSort/2026-05-07/no_extension/README
```

`ext_then_date`：

```text
~/FileAutoSort/pdf/2026-05-07/demo.pdf
~/FileAutoSort/png/2026-05-07/image.png
```

## Release

发布步骤详见 [RELEASING.md](/Users/life2you/vibeCodes/github/autosort-mac/RELEASING.md)。

## 注意事项

- 第一次运行可能需要 macOS 授权访问 Desktop 和 Downloads。
- 下载中的文件不会立即移动。
- 被其他应用打开中的文件不会立即移动。
- 如果文件仍在写入，会等待稳定后再移动。
- launchd 服务日志在：

```text
/tmp/mac-file-auto-sort.log
/tmp/mac-file-auto-sort.error.log
```
