# Excalibur CLI

一个基于 Rust 和 ratatui 构建的命令行历史管理工具，为 Fish Shell 提供强大的交互式历史命令搜索。

## 功能特性

### 🏗️ 模块化架构
- 可扩展的模块系统
- 独立模块管理
- 主菜单导航系统

### 📜 命令历史模块
- **Fish Shell 集成**: 与 Fish Shell 无缝集成，类似 fzf
- **历史解析**: 读取和解析 `~/.local/share/fish/fish_history`
- **统计分析**: 自动聚合相同命令并统计使用次数
- **多种排序**:
  - 按使用次数排序（默认）
  - 按最近使用时间排序
  - 按字母顺序排序
- **实时搜索**: 大小写不敏感的实时过滤
- **详情展示**: 显示首次/最后使用时间、总使用次数
- **虚拟滚动**: 支持大型历史文件（10,000+ 命令）无卡顿
- **命令输出**: 按 Enter 将命令输出到 shell 命令行

## 快速开始

### 安装

```bash
# 1. 克隆仓库
git clone https://github.com/yourusername/excalibur-cli
cd excalibur-cli/excalibur

# 2. 构建并安装
cargo build --release
cargo install --path .

# 3. 安装 Fish Shell 集成
cd install
cp exh.fish ~/.config/fish/functions/

# 4. 重载配置
source ~/.config/fish/config.fish
```

### 使用方法

#### 方式 1: 快捷键（推荐）

在 Fish Shell 中按 `Ctrl+R` 启动 Excalibur。

#### 方式 2: 命令

```bash
# 使用 exh (excalibur history) 命令
exh
```

#### 方式 3: 直接运行二进制

```bash
# 直接运行（独立模式，不自动插入命令）
excalibur
```

## 快捷键

### 主菜单
- `h` - 进入命令历史模块
- `↑/↓` 或 `j/k` - 导航
- `Enter` - 选择
- `q` / `Esc` / `Ctrl+C` - 退出

### 命令历史模块

**普通模式**:
- `Enter` - 选中命令并插入到 shell（可以编辑后再执行）
- `Ctrl+O` - 选中命令并**立即执行**（无需确认）
- `q` / `Esc` - 返回主菜单
- `/` - 进入搜索模式
- `s` - 切换排序方式
- `y` - 复制命令到剪贴板（可选）
- `↑/↓` / `j/k` - 上下导航
- `PageUp/PageDown` - 快速翻页
- `g` / `G` - 跳到首行/末行

**搜索模式**:
- `Esc` - 退出搜索并清空
- `Enter` - 退出搜索保持过滤
- `Backspace` - 删除字符
- 输入字符 - 实时过滤

## Fish Shell 集成原理

Excalibur 采用与 `fzf`、`zoxide` 等工具一致的集成方式：

```
用户按 Ctrl+R
  ↓
Fish 调用 excalibur 函数 (~/.config/fish/functions/excalibur.fish)
  ↓
函数运行 excalibur 二进制程序
  ↓
用户在 TUI 中选择命令，按 Enter
  ↓
程序将命令输出到 stdout 并退出
  ↓
Fish 函数捕获输出，使用 commandline -r 插入到命令行
  ↓
用户可以编辑命令后执行
```

**关键点**：
- ✅ 不修改 Fish 源代码
- ✅ 用户级配置，完全可控
- ✅ 可随时卸载（删除函数文件即可）
- ✅ 遵循 Fish 生态标准做法

详细说明见 [install/README.md](install/README.md)。

## 技术栈

- **语言**: Rust (edition 2024)
- **TUI**: ratatui 0.29
- **终端**: crossterm 0.28
- **解析**: serde_yaml (Fish history YAML 格式)
- **其他**: chrono, arboard, dirs

## 性能优化

- **预加载**: 启动时一次性加载所有历史，避免重复文件 I/O
- **虚拟滚动**: 只渲染可见行（约 30 行），支持 10,000+ 命令流畅操作
- **Unicode 安全**: 正确处理多字节字符（中文、日文等）
- **30 FPS**: 固定帧率确保流畅动画

## 开发文档

详细的架构设计和开发指南请参阅 [DEVELOPMENT.md](DEVELOPMENT.md)。

## 卸载

```bash
# 卸载二进制
cargo uninstall excalibur

# 删除 Fish 集成
rm ~/.config/fish/functions/excalibur.fish

# 删除快捷键绑定（从 config.fish 中删除 bind \cr excalibur 行）
```

## License

Copyright (c) lxb <liuxiaobo666233@gmail.com>

This project is licensed under the MIT license ([LICENSE](./LICENSE) or <http://opensource.org/licenses/MIT>)
