# Fish Shell 集成 - 使用 exh 命令

## 快速开始

### 安装

```fish
# 1. 复制函数文件
cp install/exh.fish ~/.config/fish/functions/

# 2. 重载配置
source ~/.config/fish/config.fish
```

### 使用

```fish
# 方式 1: 命令
exh

# 方式 2: 快捷键（已绑定）
# 按 Ctrl+R
```

## 为什么叫 exh？

- **`exh`** = **Ex**calibur **H**istory
- 简短易记
- 避免与二进制文件名 `excalibur` 冲突

## 功能

在 Fish shell 中：

1. **`exh`** - 启动交互式历史浏览器
   - 按 `Enter` → 命令插入到命令行（可编辑）
   - 按 `Ctrl+O` → 命令立即执行

2. **`Ctrl+R`** - 快捷键（自动绑定到 `exh`）

3. **`excalibur`** - 直接运行二进制（独立模式）

## 验证安装

```fish
# 检查函数是否加载
functions exh

# 测试
exh
```

## 卸载

```fish
rm ~/.config/fish/functions/exh.fish
source ~/.config/fish/config.fish
```

## 技术说明

- Fish 函数调用 `command excalibur` 运行二进制
- 使用 `/dev/tty` 进行 TUI 渲染（不会被 command substitution 缓冲）
- 通过 stdout 捕获选中的命令
- 根据 exit code 决定是否自动执行（0=插入，10=执行）
