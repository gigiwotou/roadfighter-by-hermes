# Road Fighter (火箭车) 游戏开发完整记录

**项目路径**: `/home/yinming/speedcar/road-fighter/`  
**开发语言**: Rust  
**图形库**: minifb (最终版本)  
**创建日期**: 2026年4月14日

---

## 目录

1. [项目概述](#项目概述)
2. [开发历程](#开发历程)
3. [技术决策](#技术决策)
4. [最终实现](#最终实现)
5. [问题与解决方案](#问题与解决方案)
6. [当前状态](#当前状态)

---

## 项目概述

Road Fighter（火箭车）是任天堂FC上的经典赛车游戏。用户要求使用Rust开发一个复刻版本。

**核心玩法**:
- 控制赛车在4车道公路上行驶
- 躲避其他车辆
- 收集燃料补给
- 避免燃料耗尽
- 3条生命，碰撞扣除生命

---

## 开发历程

### 第一阶段：项目创建 (2026-04-14 16:45)

**用户需求**: 在 `/home/yinming/speedcar` 目录下创建 Rust 版 Road Fighter 游戏。

**初始环境检查**:
- 目录为空
- Cargo 未安装
- 需要 Rust 工具链

**安装 Rust**:
```bash
sudo apt install rustc cargo
# 安装版本: rustc 1.85, cargo 1.85
```

### 第二阶段：Macroquad 实现

**初始选择**: 使用 `macroquad` 游戏引擎

**原因**: macroquad 是流行的 Rust 2D 游戏引擎，支持跨平台

**分辨率调整历程**:
1. 初始: 800x600
2. 调整: 1024x768 (用户反馈显示问题)
3. 调整: 1280x960
4. 回退: 800x600

**遇到的问题**:
- 显示不正常
- 输入响应问题
- 窗口定位问题

### 第三阶段：框架迁移 (2026-04-14)

**决策**: 从 macroquad 迁移到 minifb

**原因**:
- macroquad 在用户环境存在兼容性问题
- minifb 更简单，使用纯像素缓冲区渲染
- 更适合复古像素风格游戏

**迁移工作**:
- 重写所有绘图代码
- 实现像素级渲染函数
- 使用 `Vec<u32>` 作为帧缓冲区

### 第四阶段：窗口居中 (2026-04-15 14:50)

**问题**: 游戏窗口始终出现在屏幕左上角，无法居中

**排查过程**:
1. 检测到系统使用 Wayland 显示协议
2. 发现 minifb 的 `set_position()` 在 Wayland 下无效
3. Wayland 协议不支持服务端窗口定位

**解决方案**:
```rust
// 强制使用 X11 后端
env::set_var("WAYLAND_DISPLAY", "");
```

**屏幕尺寸检测优化**:
- 优先使用 `xrandr` 获取实际显示器分辨率
- 备用方案使用 `xdpyinfo`
- 默认回退 1920x1080

```rust
fn get_screen_size() -> (u32, u32) {
    // 优先使用 xrandr
    if let Ok(output) = Command::new("xrandr").output() {
        // 解析 "Virtual-1 connected primary 4096x2560+0+0"
        ...
    }
    // 备用: xdpyinfo
    if let Ok(output) = Command::new("xdpyinfo").output() {
        // 解析 "dimensions: 4096x2560 pixels"
        ...
    }
    (1920, 1080)
}
```

### 第五阶段：语法错误修复 (2026-04-15 15:03)

**问题**: 用户报告"最新版本会让系统崩溃"

**排查**:
```bash
cargo build --release
# 错误: unexpected closing delimiter: `}`
# --> src/main.rs:96:1
```

**原因**: `get_screen_size()` 函数末尾有多余的 `}`

**修复**: 删除第96行多余的 `}`

---

## 技术决策

### 图形库选择

| 特性 | macroquad | minifb |
|------|-----------|--------|
| 渲染方式 | GPU 加速 | CPU 软件渲染 |
| 复杂度 | 中等 | 简单 |
| 依赖 | 较多 | 较少 |
| 跨平台 | 优秀 | 良好 |
| 适合复古风 | 一般 | 优秀 |

**最终选择**: minifb

### 渲染机制

```
┌─────────────────────────────────────────┐
│           minifb 渲染流程               │
├─────────────────────────────────────────┤
│  1. 创建 Vec<u32> 像素缓冲区            │
│     (800 * 600 = 480,000 像素)          │
│                                         │
│  2. 每帧清空缓冲区                      │
│                                         │
│  3. 绘制草地、道路、车辆等              │
│     - draw_rect() 填充矩形              │
│     - draw_text() 位图字体              │
│                                         │
│  4. window.update_with_buffer()        │
│     将缓冲区送到显示器                  │
└─────────────────────────────────────────┘
```

---

## 最终实现

### 项目结构

```
/home/yinming/speedcar/road-fighter/
├── Cargo.toml
├── src/
│   └── main.rs      (702 行)
└── target/
    └── release/
        └── road-fighter
```

### Cargo.toml

```toml
[package]
name = "road-fighter"
version = "0.1.0"
edition = "2021"

[dependencies]
minifb = "0.27"
rand = "0.8"

[profile.release]
opt-level = 3
```

### 核心常量

```rust
const SCREEN_WIDTH: usize = 800;
const SCREEN_HEIGHT: usize = 600;
const ROAD_WIDTH: f32 = 360.0;
const LANE_COUNT: usize = 4;
```

### 游戏对象尺寸

| 对象 | 尺寸 (像素) | 颜色 |
|------|------------|------|
| 玩家赛车 | 36 x 60 | 蓝色 |
| 敌方车辆 | 32 x 52 | 红/黄/紫/橙 |
| 燃料补给 | 28 x 18 | 绿色 |

### 游戏状态机

```rust
enum GameState {
    Title,      // 标题画面
    Playing,    // 游戏进行中
    GameOver,   // 游戏结束
}
```

### 绘图函数

```rust
// 矩形填充 (核心绘图原语)
fn draw_rect(buffer: &mut Vec<u32>, x: f32, y: f32, 
             w: f32, h: f32, color: u32)

// 赛车绘制
fn draw_car(buffer: &mut Vec<u32>, x: f32, y: f32, 
            color: u32, is_player: bool)

// 文字绘制 (位图字体)
fn draw_text(buffer: &mut Vec<u32>, text: &str, 
             x: i32, y: i32, scale: i32, color: u32)
```

### 控制方案

| 按键 | 功能 |
|------|------|
| ↑ / W | 加速/向上移动 |
| ↓ / S | 减速/向下移动 |
| ← / A | 左移 |
| → / D | 右移 |
| Space | 开始游戏/继续 |
| ESC | 退出 |

---

## 问题与解决方案

### 问题 1: 显示异常 (macroquad)

**现象**: 使用 macroquad 时显示不正常

**尝试方案**:
- 调整分辨率
- 修改窗口选项

**最终方案**: 迁移到 minifb

### 问题 2: 窗口无法居中

**现象**: 游戏窗口始终在左上角

**原因**: Wayland 协议不支持服务端窗口定位

**解决方案**: 
```rust
env::set_var("WAYLAND_DISPLAY", "");
```
强制使用 X11/XWayland 后端

### 问题 3: 编译错误 "系统崩溃"

**现象**: 用户报告"最新版本会让系统崩溃"

**实际原因**: 编译失败，语法错误

**错误信息**:
```
error: unexpected closing delimiter: `}`
 --> src/main.rs:96:1
```

**解决**: 删除多余的 `}`

---

## 当前状态

### 编译状态
```
Compiling road-fighter v0.1.0
Finished `release` profile [optimized]
```

**警告** (不影响运行):
- `get_screen_width` 未使用
- `get_screen_height` 未使用  
- `FONT_DATA` 未使用

### 运行状态

```bash
# 运行命令
cd /home/yinming/speedcar/road-fighter
WAYLAND_DISPLAY="" ./target/release/road-fighter

# 或直接
cargo run --release
```

### 已实现功能

- [x] 标题画面
- [x] 4车道公路
- [x] 玩家控制赛车
- [x] 敌方车辆生成
- [x] 碰撞检测
- [x] 燃料系统
- [x] 燃料补给收集
- [x] 生命值系统 (3条命)
- [x] 分数系统
- [x] 关卡系统
- [x] 最高分记录
- [x] 窗口居中
- [x] 位图字体渲染

### 待开发功能

- [ ] 音效
- [ ] 背景音乐
- [ ] 更多关卡设计
- [ ] 敌人 AI 改进
- [ ] 特殊道具
- [ ] 存档功能

---

## 附录：颜色定义

```rust
const BLACK: u32 = 0x000000;       // 黑色
const WHITE: u32 = 0xFFFFFF;       // 白色
const RED: u32 = 0xFF0000;         // 红色
const GREEN: u32 = 0x00FF00;       // 绿色
const BLUE: u32 = 0x0000FF;        // 蓝色
const YELLOW: u32 = 0xFFFF00;      // 黄色
const GRAY: u32 = 0x808080;        // 灰色 (道路)
const DARK_GRAY: u32 = 0x404040;   // 深灰 (背景)
const GRASS_GREEN: u32 = 0x228B22; // 草地绿
const ORANGE: u32 = 0xFFA500;      // 橙色
const PURPLE: u32 = 0x800080;      // 紫色
const GOLD: u32 = 0xFFD700;        // 金色
const SKY_BLUE: u32 = 0x87CEEB;    // 天蓝 (车窗)
```

---

## 相关会话记录

| 会话 ID | 时间 | 内容 |
|---------|------|------|
| 20260414_184411_4ab92b | 2026-04-14 16:45 | 项目创建，macroquad 实现 |
| 20260414_235846_3d2472 | 2026-04-14 23:58 | 查看对话记录 |
| 20260414_235924_379a80 | 2026-04-14 23:59 | 查看对话记录 |
| 20260415_145007_94eb0d | 2026-04-15 14:50 | 窗口居中修复 |
| 20260415_150307_* | 2026-04-15 15:03 | 语法错误修复 |

---

*文档生成时间: 2026-04-15*  
*最后更新: 语法错误修复完成*
