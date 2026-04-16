# Road Fighter by Hermes

> 任天堂FC经典赛车游戏克隆 - 由 Hermes AI 开发

![Rust](https://img.shields.io/badge/Rust-1.85+-orange?logo=rust)
![Platform](https://img.shields.io/badge/Platform-Linux-blue)
![License](https://img.shields.io/badge/License-MIT-green)

## 游戏简介

Road Fighter（火箭车）是Konami于1984年在任天堂FC平台上发行的经典赛车游戏。本项目使用 Rust 语言完整复刻了这款怀旧经典。

### 游戏特色

- 4车道垂直滚动赛车玩法
- 躲避障碍车辆，收集燃料补给
- 生命系统与燃料管理
- 分数累计与关卡递进
- 难度随关卡提升

## 技术栈

- **语言**: Rust 1.85+
- **图形库**: minifb 0.27 (像素级渲染)
- **随机数**: rand 0.8

## 快速开始

### 环境要求

- Rust 工具链 (`rustc`, `cargo`)
- Linux 环境 (已测试)
- X11 显示服务器 (Wayland 用户需设置 `WAYLAND_DISPLAY=""`)

### 编译运行

```bash
# 克隆仓库
git clone https://github.com/你的用户名/roadfighter-by-hermes.git
cd roadfighter-by-hermes

# 编译 (Release 模式)
cargo build --release

# 运行
WAYLAND_DISPLAY="" ./target/release/roadfighter-by-hermes
```

## 游戏操作

| 按键 | 功能 |
|------|------|
| `Space` | 开始游戏 / 继续 |
| `↑` / `W` | 向上移动 |
| `↓` / `S` | 向下移动 |
| `←` / `A` | 向左移动 |
| `→` / `D` | 向右移动 |
| `ESC` | 退出游戏 |

## 游戏截图

```
┌────────────────────────────────────────┐
│  FUEL ████████████░░░  SCORE:1250      │
│  LV:3  HIGH:4500                       │
│                                        │
│    ████████████████████████████       │
│    █                      █           │
│    █   ╔════════════╗    █  草地      │
│    █   ║  ╔═══╗     ║    █           │
│    █   ║  │▓▓▓│  🚗 ║    █  道路      │
│    █   ║  ╚═══╝     ║    █           │
│    █   ╚════════════╝    █           │
│    █                      █           │
│    ████████████████████████████       │
│                                        │
│         🚙 敌车    ⛽ 燃料补给         │
│              🚗 玩家                  │
└────────────────────────────────────────┘
```

## 项目结构

```
roadfighter-by-hermes/
├── Cargo.toml          # 项目配置
├── README.md           # 说明文档
├── conversation_export.md  # 开发对话记录
├── src/
│   └── main.rs         # 游戏主代码 (~700行)
└── target/             # 编译输出
```

## 开发历程

本项目由 Hermes AI 在与用户的对话中逐步开发完成，完整开发对话记录见 [`conversation_export.md`](conversation_export.md)。

### 主要里程碑

1. 环境搭建 - 安装 Rust/Cargo
2. 初始实现 - 使用 macroquad 引擎
3. 框架迁移 - 切换到 minifb 像素渲染
4. 窗口居中 - 解决 Wayland 兼容性问题
5. 位图字体 - 实现 5x7 像素字体渲染

## 致谢

- 原版游戏: Konami - Road Fighter (1984)
- 开发工具: Hermes AI Agent

## 许可证

MIT License

---

*Made with ❤️ by Hermes AI*
