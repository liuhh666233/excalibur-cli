# Excalibur CLI - 开发文档

## 目录
- [1. 架构概述](#1-架构概述)
- [2. 核心组件](#2-核心组件)
- [3. 数据流和事件流](#3-数据流和事件流)
- [4. 模块系统详解](#4-模块系统详解)
- [5. 添加新模块指南](#5-添加新模块指南)
- [6. 关键设计决策](#6-关键设计决策)
- [7. 代码规范](#7-代码规范)

---

## 1. 架构概述

### 1.1 整体架构

Excalibur CLI 采用**模块化事件驱动架构**，核心设计理念是：

```
┌─────────────────────────────────────────────────────────────┐
│                        main.rs                              │
│  - 初始化 color_eyre 错误处理                               │
│  - 初始化 ratatui 终端                                       │
│  - 创建并运行 App                                           │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                         App                                 │
│  ┌───────────────────────────────────────────────────────┐ │
│  │  - running: bool         (运行状态)                   │ │
│  │  - current_view: View    (当前视图)                   │ │
│  │  - module_manager        (模块管理器)                 │ │
│  │  - selected_menu_item    (菜单选择索引)               │ │
│  │  - events                (事件处理器)                 │ │
│  └───────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
            │                    │                    │
            │                    │                    │
    ┌───────▼───────┐    ┌──────▼──────┐    ┌───────▼────────┐
    │  View Enum    │    │  EventHandler│    │ ModuleManager │
    │               │    │               │    │               │
    │ - MainMenu    │    │ - Tick (30fps)│    │ - modules: Map│
    │ - Module(ID)  │    │ - Crossterm   │    │ - active: ID  │
    │               │    │ - App Events  │    │               │
    └───────────────┘    └───────────────┘    └───────┬────────┘
                                                       │
                                              ┌────────▼────────┐
                                              │  Module Trait   │
                                              │                 │
                                              │  - metadata()   │
                                              │  - init()       │
                                              │  - handle_key() │
                                              │  - update()     │
                                              │  - render()     │
                                              │  - cleanup()    │
                                              └────────┬────────┘
                                                       │
                                              ┌────────▼────────┐
                                              │ HistoryModule   │
                                              │                 │
                                              │  - state        │
                                              │  - parser       │
                                              │  - clipboard    │
                                              └─────────────────┘
```

### 1.2 目录结构

```
excalibur/
├── Cargo.toml                  # 项目配置和依赖
├── README.md                   # 用户文档
├── DEVELOPMENT.md              # 开发文档（本文件）
└── src/
    ├── main.rs                 # 应用入口点
    ├── app.rs                  # App 核心逻辑
    ├── event.rs                # 事件系统
    ├── ui.rs                   # 主菜单 UI
    ├── view.rs                 # 视图枚举
    └── modules/                # 功能模块
        ├── mod.rs              # Module trait 定义
        ├── manager.rs          # ModuleManager 实现
        └── history/            # 历史命令模块
            ├── mod.rs          # 模块主入口
            ├── parser.rs       # Fish 历史解析
            ├── state.rs        # 状态管理
            ├── ui.rs           # UI 渲染
            └── clipboard.rs    # 剪贴板功能
```

---

## 2. 核心组件

### 2.1 main.rs - 应用入口

**职责**：初始化和启动应用

```rust
pub mod app;
pub mod event;
pub mod modules;
pub mod ui;
pub mod view;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;        // 安装错误处理
    let terminal = ratatui::init(); // 初始化终端
    let result = App::new().run(terminal); // 运行应用
    ratatui::restore();             // 恢复终端
    result
}
```

**关键点**：
- 模块声明顺序很重要
- 错误处理使用 `color_eyre`
- 终端初始化和清理使用 `ratatui` 的全局函数

### 2.2 app.rs - 应用核心

#### App 结构体

```rust
pub struct App {
    pub running: bool,                    // 应用运行状态
    pub current_view: View,               // 当前视图
    pub module_manager: ModuleManager,    // 模块管理器
    pub selected_menu_item: usize,        // 主菜单选中项
    pub events: EventHandler,             // 事件处理器
}
```

#### 核心方法

| 方法 | 职责 |
|------|------|
| `new()` | 创建新应用实例 |
| `run()` | 主事件循环 |
| `handle_events()` | 处理事件分发 |
| `handle_key_event()` | 路由键盘事件 |
| `handle_main_menu_keys()` | 主菜单键盘处理 |
| `tick()` | 定期更新（30fps） |
| `quit()` | 退出应用 |

#### 事件处理流程

```rust
handle_events() {
    match event {
        Tick => tick(),                    // 30fps 更新
        Crossterm(key) => handle_key_event(),  // 键盘输入
        App(event) => match event {
            EnterModule(id) => activate_module(id),
            ExitModule => deactivate_module(),
            ModuleAction(action) => process_action(action),
            Quit => quit(),
        }
    }
}
```

### 2.3 event.rs - 事件系统

#### 事件类型层次

```rust
// 顶层事件枚举
pub enum Event {
    Tick,                    // 定期触发（30fps）
    Crossterm(CrosstermEvent), // 终端事件
    App(AppEvent),           // 应用事件
}

// 应用级事件
pub enum AppEvent {
    EnterModule(ModuleId),   // 进入模块
    ExitModule,              // 退出模块
    ModuleAction(ModuleAction), // 模块动作
    Quit,                    // 退出应用
}
```

#### EventHandler 架构

```rust
pub struct EventHandler {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
}

// 后台线程持续运行
EventThread::run() {
    loop {
        // 1. 检查是否需要发送 Tick
        if timeout == Duration::ZERO {
            send(Event::Tick);
        }

        // 2. 非阻塞检查 Crossterm 事件
        if event::poll(timeout)? {
            let event = event::read()?;
            send(Event::Crossterm(event));
        }
    }
}
```

**关键设计**：
- 使用独立线程处理事件，避免阻塞主循环
- 30 FPS 的固定帧率确保流畅动画
- `mpsc` 通道实现线程安全通信

### 2.4 view.rs - 视图管理

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum View {
    MainMenu,          // 主菜单视图
    Module(ModuleId),  // 模块视图
}
```

**视图切换逻辑**：
```
MainMenu ─[按 h/Enter]→ Module(History) ─[按 Esc/q]→ MainMenu
```

### 2.5 ui.rs - 主菜单渲染

#### Widget 实现

```rust
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match &self.current_view {
            View::MainMenu => self.render_main_menu(area, buf),
            View::Module(_) => self.render_module(area, buf),
        }
    }
}
```

#### 主菜单布局

```rust
Layout::vertical([
    Constraint::Length(3),  // 标题栏
    Constraint::Min(0),     // 模块列表
    Constraint::Length(3),  // 帮助栏
])
```

---

## 3. 数据流和事件流

### 3.1 应用启动流程

```
1. main()
   ├─ color_eyre::install()
   ├─ ratatui::init()
   └─ App::new()
      ├─ View::MainMenu
      ├─ ModuleManager::new()
      │  └─ 注册所有模块
      └─ EventHandler::new()
         └─ 启动事件线程

2. App::run()
   └─ loop while running {
      ├─ terminal.draw(|frame| render)
      └─ handle_events()
   }
```

### 3.2 用户交互流程

#### 场景 1：进入历史命令模块

```
1. 用户按下 'h'
   ├─ Crossterm 捕获键盘事件
   └─ EventThread::send(Crossterm(KeyEvent))

2. App::handle_events()
   ├─ 识别为键盘事件
   └─ App::handle_main_menu_keys()
      ├─ 匹配快捷键 'h'
      └─ events.send(EnterModule(History))

3. 下一轮事件循环
   ├─ 收到 App(EnterModule(History))
   ├─ module_manager.activate(History)
   │  └─ HistoryModule::init()
   │     ├─ 解析 Fish history 文件
   │     └─ 创建 HistoryState
   └─ current_view = Module(History)

4. 渲染循环
   └─ module_manager.render()
      └─ HistoryModule::render()
```

#### 场景 2：搜索命令

```
1. 用户在历史模块按 '/'
   ├─ handle_key_event() → module_manager.handle_key_event()
   └─ HistoryModule::handle_normal_mode()
      └─ state.input_mode = Search

2. 用户输入 "git"
   ├─ 每个字符触发 Char 事件
   ├─ HistoryModule::handle_search_mode()
   │  ├─ state.search_query.push(char)
   │  └─ state.apply_filters()
   │     ├─ 过滤命令列表
   │     ├─ 应用排序
   │     └─ 更新 table_state
   └─ 下一帧渲染更新的列表
```

### 3.3 模块生命周期

```
┌─────────────────────────────────────────────────────────┐
│                     模块生命周期                         │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  1. 创建                                                │
│     ModuleManager::new()                                │
│     └─ HistoryModule::new()  (创建但未初始化)           │
│                                                         │
│  2. 激活                                                │
│     module_manager.activate(ModuleId::History)          │
│     └─ module.init()                                    │
│        ├─ 加载数据                                      │
│        ├─ 初始化状态                                     │
│        └─ 准备 UI                                       │
│                                                         │
│  3. 运行中                                              │
│     ├─ handle_key_event() - 每次按键                    │
│     ├─ update() - 每帧 (30fps)                         │
│     └─ render() - 每帧                                  │
│                                                         │
│  4. 停用                                                │
│     module_manager.deactivate()                         │
│     └─ module.cleanup()                                 │
│        ├─ 释放资源                                      │
│        └─ 重置状态                                      │
│                                                         │
│  5. 可重新激活（返回步骤 2）                             │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## 4. 模块系统详解

### 4.1 Module Trait 接口

```rust
pub trait Module: std::fmt::Debug {
    /// 返回模块元数据（名称、描述、快捷键）
    fn metadata(&self) -> ModuleMetadata;

    /// 模块初始化（进入时调用一次）
    fn init(&mut self) -> Result<()>;

    /// 处理键盘事件
    fn handle_key_event(&mut self, key_event: KeyEvent)
        -> Result<ModuleAction>;

    /// 定期更新（30fps）
    fn update(&mut self) -> Result<()>;

    /// 渲染 UI
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// 清理资源（退出时调用）
    fn cleanup(&mut self) -> Result<()>;
}
```

**设计考虑**：
- `Debug` 约束：便于调试和错误追踪
- `init/cleanup` 对称设计：资源管理清晰
- `update/render` 分离：逻辑和渲染解耦
- `Result` 返回：统一错误处理

### 4.2 ModuleMetadata 结构

```rust
pub struct ModuleMetadata {
    pub id: ModuleId,              // 唯一标识符
    pub name: String,              // 显示名称
    pub description: String,       // 描述文本
    pub shortcut: Option<char>,    // 快捷键
}
```

### 4.3 ModuleAction 枚举

```rust
pub enum ModuleAction {
    None,                    // 无操作
    Exit,                    // 退出模块（返回主菜单）
    Quit,                    // 退出整个应用
    Notification(String),    // 显示通知
}
```

**使用模式**：
```rust
fn handle_key_event(&mut self, key: KeyEvent) -> Result<ModuleAction> {
    match key.code {
        KeyCode::Esc => Ok(ModuleAction::Exit),
        KeyCode::Char('y') => {
            // 复制成功
            Ok(ModuleAction::Notification("Copied!".into()))
        }
        _ => Ok(ModuleAction::None)
    }
}
```

### 4.4 ModuleManager 实现

#### 核心数据结构

```rust
pub struct ModuleManager {
    modules: HashMap<ModuleId, Box<dyn Module>>,
    active_module: Option<ModuleId>,
}
```

**关键点**：
- 使用 `HashMap` 存储所有模块
- `Box<dyn Module>` 实现动态分发
- `active_module` 追踪当前活跃模块

#### 核心方法

```rust
impl ModuleManager {
    // 注册所有模块
    pub fn new() -> Self {
        let mut modules = HashMap::new();
        modules.insert(
            ModuleId::History,
            Box::new(HistoryModule::new()) as Box<dyn Module>
        );
        // 未来添加更多模块...
        Self { modules, active_module: None }
    }

    // 激活模块
    pub fn activate(&mut self, id: ModuleId) -> Result<()> {
        if let Some(module) = self.modules.get_mut(&id) {
            module.init()?;
            self.active_module = Some(id);
        }
        Ok(())
    }

    // 路由事件到活跃模块
    pub fn handle_key_event(&mut self, key: KeyEvent)
        -> Result<ModuleAction> {
        if let Some(module) = self.get_active_mut() {
            module.handle_key_event(key)
        } else {
            Ok(ModuleAction::None)
        }
    }
}
```

---

## 5. 添加新模块指南

### 5.1 快速开始

假设我们要添加一个 **Git 模块**来管理 Git 仓库。

#### 步骤 1：定义模块 ID

编辑 `src/modules/mod.rs`：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModuleId {
    History,
    Git,  // 新增
}
```

#### 步骤 2：创建模块目录

```bash
mkdir -p src/modules/git
```

#### 步骤 3：创建模块结构

`src/modules/git/mod.rs`：

```rust
use super::{Module, ModuleAction, ModuleId, ModuleMetadata};
use color_eyre::Result;
use ratatui::{buffer::Buffer, crossterm::event::KeyEvent, layout::Rect};

#[derive(Debug)]
pub struct GitModule {
    // 状态字段
}

impl GitModule {
    pub fn new() -> Self {
        Self { /* 初始化字段 */ }
    }
}

impl Module for GitModule {
    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata {
            id: ModuleId::Git,
            name: "Git Manager".to_string(),
            description: "Manage Git repositories".to_string(),
            shortcut: Some('g'),
        }
    }

    fn init(&mut self) -> Result<()> {
        // 加载 Git 仓库信息
        Ok(())
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        // 处理键盘事件
        Ok(ModuleAction::None)
    }

    fn update(&mut self) -> Result<()> {
        // 更新状态
        Ok(())
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        // 渲染 UI
    }

    fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}
```

#### 步骤 4：注册模块

编辑 `src/modules/mod.rs`：

```rust
pub mod git;  // 新增
pub mod history;
pub mod manager;
```

编辑 `src/modules/manager.rs`：

```rust
use super::{git::GitModule, history::HistoryModule, ...};

impl ModuleManager {
    pub fn new() -> Self {
        let mut modules = HashMap::new();

        modules.insert(
            ModuleId::History,
            Box::new(HistoryModule::new())
        );

        // 新增 Git 模块
        modules.insert(
            ModuleId::Git,
            Box::new(GitModule::new())
        );

        Self { modules, active_module: None }
    }
}
```

#### 步骤 5：测试

```bash
cargo build
cargo run
```

现在主菜单中应该会显示新的 Git 模块！

### 5.2 模块开发最佳实践

#### 5.2.1 模块内部结构建议

```
src/modules/your_module/
├── mod.rs              # 模块入口，实现 Module trait
├── state.rs            # 状态管理（推荐）
├── ui.rs               # UI 渲染逻辑
├── data.rs             # 数据模型
└── operations.rs       # 业务逻辑
```

#### 5.2.2 状态管理模式

```rust
// state.rs
pub struct YourModuleState {
    // UI 状态
    pub selected_index: usize,
    pub table_state: TableState,

    // 数据
    pub items: Vec<Item>,
    pub filtered_items: Vec<usize>,

    // 交互状态
    pub input_mode: InputMode,
    pub search_query: String,
}

impl YourModuleState {
    pub fn new(items: Vec<Item>) -> Self { /* ... */ }
    pub fn select_next(&mut self) { /* ... */ }
    pub fn apply_filter(&mut self) { /* ... */ }
}
```

#### 5.2.3 UI 渲染模式

```rust
// ui.rs
pub fn render(state: &YourModuleState, area: Rect, buf: &mut Buffer) {
    let chunks = Layout::vertical([
        Constraint::Length(3),   // Header
        Constraint::Min(0),      // Main content
        Constraint::Length(3),   // Footer
    ]).split(area);

    render_header(state, chunks[0], buf);
    render_content(state, chunks[1], buf);
    render_footer(state, chunks[2], buf);
}

fn render_header(state: &YourModuleState, area: Rect, buf: &mut Buffer) {
    // 渲染标题栏
}
```

#### 5.2.4 键盘处理模式

```rust
// mod.rs
impl YourModule {
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        match self.state.input_mode {
            InputMode::Normal => self.handle_normal_mode(key),
            InputMode::Edit => self.handle_edit_mode(key),
        }
    }

    fn handle_normal_mode(&mut self, key: KeyEvent) -> Result<ModuleAction> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                Ok(ModuleAction::Exit)
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.state.select_previous();
                Ok(ModuleAction::None)
            }
            // 更多按键...
            _ => Ok(ModuleAction::None)
        }
    }
}
```

---

## 6. 关键设计决策

### 6.1 为什么使用 Trait Object？

```rust
HashMap<ModuleId, Box<dyn Module>>
```

**优点**：
- 运行时动态分发
- 支持异构模块集合
- 易于添加新模块

**缺点**：
- 轻微的运行时开销（可忽略）
- 不支持泛型方法

**替代方案**：
- 枚举：需要在编译时知道所有模块
- 泛型：复杂度高，不适合动态模块系统

### 6.2 为什么分离 update() 和 render()？

```rust
fn update(&mut self) -> Result<()>;  // 可变借用
fn render(&self, ...) -> Result<()>; // 不可变借用
```

**原因**：
1. **关注点分离**：逻辑更新和视觉呈现分开
2. **借用检查**：`render` 不需要修改状态
3. **性能**：`render` 可以在多处调用而不影响状态
4. **测试**：可以独立测试更新逻辑

### 6.3 为什么使用 30 FPS？

```rust
const TICK_FPS: f64 = 30.0;
```

**考虑**：
- **平衡**：足够流畅 vs 不浪费 CPU
- **人眼感知**：24-30fps 已经很流畅
- **终端刷新**：大多数终端 ~60Hz
- **电池**：降低功耗

### 6.4 为什么 Module.render() 使用 Buffer？

```rust
fn render(&self, area: Rect, buf: &mut Buffer);
```

而不是：
```rust
fn render(&self, frame: &mut Frame);
```

**原因**：
1. **生命周期简化**：`Buffer` 更简单
2. **灵活性**：可以渲染到任意 `Buffer`
3. **测试友好**：易于创建测试 `Buffer`
4. **ratatui 设计**：`Widget` trait 的标准模式

---

## 7. 代码规范

### 7.1 命名约定

| 类型 | 约定 | 示例 |
|------|------|------|
| 模块 | snake_case | `history`, `git_manager` |
| 结构体 | PascalCase | `HistoryModule`, `GitState` |
| 枚举 | PascalCase | `ModuleId`, `SortMode` |
| 函数 | snake_case | `handle_key_event`, `apply_filters` |
| 常量 | UPPER_SNAKE_CASE | `TICK_FPS`, `MAX_ITEMS` |
| 类型别名 | PascalCase | `Result<T>` |

### 7.2 文件组织

#### 小模块（< 500 行）
```
src/modules/small_module.rs
```

#### 大模块（> 500 行）
```
src/modules/large_module/
├── mod.rs       # 公共接口
├── state.rs     # 状态管理
├── ui.rs        # UI 渲染
└── utils.rs     # 辅助函数
```

### 7.3 错误处理

**使用 color_eyre::Result**：
```rust
use color_eyre::Result;

pub fn parse_data() -> Result<Data> {
    let content = std::fs::read_to_string(path)?;
    let data = serde_yaml::from_str(&content)?;
    Ok(data)
}
```

**自定义错误消息**：
```rust
use color_eyre::eyre::{eyre, WrapErr};

// 使用 eyre! 创建错误
return Err(eyre!("Failed to find config file"));

// 使用 wrap_err 添加上下文
std::fs::read_to_string(path)
    .wrap_err("Failed to read history file")?;
```

### 7.4 注释规范

```rust
/// 文档注释：描述公共 API
///
/// # Arguments
/// * `key` - 键盘事件
///
/// # Returns
/// 模块动作
///
/// # Example
/// ```
/// let action = module.handle_key_event(key)?;
/// ```
pub fn handle_key_event(&mut self, key: KeyEvent) -> Result<ModuleAction> {
    // 实现注释：解释复杂逻辑
    match key.code {
        // TODO: 添加更多快捷键
        KeyCode::Char('q') => Ok(ModuleAction::Exit),
        _ => Ok(ModuleAction::None)
    }
}
```

### 7.5 导入顺序

```rust
// 1. 标准库
use std::collections::HashMap;
use std::path::PathBuf;

// 2. 外部 crate
use color_eyre::Result;
use ratatui::{buffer::Buffer, layout::Rect};

// 3. 本地模块（按层级）
use crate::modules::{Module, ModuleId};
use super::state::HistoryState;
```

---

## 8. 调试技巧

### 8.1 日志记录

在 TUI 应用中不能使用 `println!`，使用文件日志：

```rust
// 添加依赖
// log = "0.4"
// env_logger = "0.11"

// main.rs
fn main() -> Result<()> {
    // 写入日志文件
    env_logger::Builder::from_default_env()
        .target(env_logger::Target::Stdout)
        .init();

    log::info!("Application started");
    // ...
}

// 模块中使用
log::debug!("Current state: {:?}", state);
log::warn!("File not found: {}", path);
```

### 8.2 暂停渲染查看终端

```rust
// 临时禁用 TUI 查看输出
ratatui::restore();
println!("Debug: {:?}", data);
std::thread::sleep(Duration::from_secs(5));
let terminal = ratatui::init();
```

### 8.3 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_filtering() {
        let mut state = HistoryState::new(vec![...]);
        state.search_query = "git".to_string();
        state.apply_filters();

        assert_eq!(state.filtered_count(), 10);
    }
}
```

---

## 9. 性能优化指南

### 9.1 避免频繁分配

**❌ 不好**：
```rust
fn render(&self, ...) {
    for item in &self.items {
        let text = format!("{}: {}", item.name, item.value); // 每次分配
        // ...
    }
}
```

**✅ 好**：
```rust
fn render(&self, ...) {
    let mut buffer = String::with_capacity(100);
    for item in &self.items {
        buffer.clear();
        write!(&mut buffer, "{}: {}", item.name, item.value);
        // ...
    }
}
```

### 9.2 懒加载数据

```rust
pub struct HistoryModule {
    state: Option<HistoryState>, // None 直到 init()
}

impl Module for HistoryModule {
    fn init(&mut self) -> Result<()> {
        // 仅在需要时加载
        self.state = Some(HistoryState::new(...));
        Ok(())
    }
}
```

### 9.3 缓存计算结果

```rust
pub struct State {
    items: Vec<Item>,
    filtered_indices: Vec<usize>,  // 缓存过滤结果
    search_query: String,
}

impl State {
    pub fn update_search(&mut self, query: String) {
        if self.search_query != query {
            self.search_query = query;
            self.apply_filters(); // 仅在改变时重新计算
        }
    }
}
```

---

## 10. 常见问题

### Q1: 如何在模块间共享数据？

**A**: 通过 `ModuleAction::Notification` 或扩展事件系统：

```rust
pub enum AppEvent {
    // ...
    DataUpdate(SharedData),
}

// 在模块中
fn handle_key_event(...) -> Result<ModuleAction> {
    // 发送数据给其他模块
    Ok(ModuleAction::DataUpdate(data))
}
```

### Q2: 如何处理异步操作？

**A**: 使用 `tokio` 或在后台线程处理：

```rust
use std::sync::{Arc, Mutex};
use std::thread;

pub struct AsyncModule {
    data: Arc<Mutex<Vec<Item>>>,
}

impl Module for AsyncModule {
    fn init(&mut self) -> Result<()> {
        let data = self.data.clone();
        thread::spawn(move || {
            // 异步加载数据
            let items = load_data_from_network();
            *data.lock().unwrap() = items;
        });
        Ok(())
    }
}
```

### Q3: 如何测试 UI 渲染？

**A**: 使用 ratatui 的测试工具：

```rust
#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn test_render() {
        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            let state = HistoryState::new(vec![...]);
            ui::render(&state, f.size(), f.buffer_mut());
        }).unwrap();

        // 验证输出
        let buffer = terminal.backend().buffer();
        assert!(buffer.content().contains("Command History"));
    }
}
```

---

## 11. 参考资源

### 官方文档
- [Ratatui 官方文档](https://ratatui.rs/)
- [Ratatui 示例](https://github.com/ratatui/ratatui/tree/main/examples)
- [Crossterm 文档](https://docs.rs/crossterm/)

### 相关项目
- [gitui](https://github.com/extrawurst/gitui) - Git TUI，架构参考
- [bottom](https://github.com/ClementTsang/bottom) - 系统监控 TUI
- [lazygit](https://github.com/jesseduffield/lazygit) - Git TUI (Go)

### 学习资源
- [Rust 异步编程](https://rust-lang.github.io/async-book/)
- [TUI 设计模式](https://ratatui.rs/concepts/)

---

## 附录 A：完整示例模块

参见 `src/modules/history/` 目录作为完整的参考实现。

## 附录 B：快捷键设计建议

| 操作 | 建议按键 | 原因 |
|------|----------|------|
| 退出/返回 | `Esc`, `q` | 直觉，常见 |
| 搜索 | `/` | Vim 风格 |
| 上下导航 | `j/k`, `↑/↓` | Vim + 通用 |
| 选择 | `Enter`, `Space` | 标准 |
| 编辑 | `e`, `i` | Vim 风格 |
| 删除 | `d`, `Delete` | Vim + 通用 |
| 复制 | `y` | Vim (yank) |
| 帮助 | `?` | 通用约定 |

---

**文档版本**: v1.0
**最后更新**: 2025-12-28
**维护者**: lxb <liuxiaobo666233@gmail.com>
