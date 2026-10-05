# Keep-Awake 保持常亮守护助手

<p align="center">
  <b>极简、超低资源消耗的跨平台屏幕常亮与休眠保护工具 (Linux & Windows)</b>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/Language-Rust-orange.svg" alt="Rust" />
  <img src="https://img.shields.io/badge/Platform-Linux%20%7C%20Windows-blue.svg" alt="Platform" />
  <img src="https://img.shields.io/badge/License-MIT-green.svg" alt="MIT License" />
  <img src="https://img.shields.io/badge/RAM-%3C%207%20MB-brightgreen.svg" alt="Memory" />
</p>

---

## 🌟 核心特性

* ⚡ **极低资源占用**：采用纯 Rust 原生开发，无 Electron、无 Python 运行时依赖，常驻内存仅 **~7 MB**（相较于常规工具节省 90%+ 内存），CPU 占用稳定在 0.0%。
* 🛡 **四重防休眠守护 (Linux)**：
  * 会话层 (`org.gnome.SessionManager.Inhibit`)
  * 屏保层 (`org.freedesktop.ScreenSaver.Inhibit`)
  * 内核/电源层 (`org.freedesktop.login1.Manager.Inhibit` 保持阻塞文件描述符，阻止待机与合盖休眠)
  * X11 屏保定时重置 (`xset s reset`)
* 🪟 **原生内核级控制 (Windows)**：
  * 通过 Win32 原生 `SetThreadExecutionState` 内核 API 直接控制电源策略，同时阻止待机、阻止显示器关闭并启用 AwayMode。
* 🔄 **随退随恢复，绝对无损**：
  * 具备完善的 RAII 与信号捕获保护（`Ctrl+C`、`SIGTERM` 或托盘退出），关闭时自动按序释放所有锁与 FD，系统原有的熄屏与休眠设定毫发无损立即生效。
* 🖥 **系统托盘与原生提示**：
  * **Linux**：原生遵循 StatusNotifierItem (KDE/Cinnamon/GNOME) 托盘规范；
  * **Windows**：原生任务栏托盘图标与 Toast 气泡提醒；
  * 左键点击快速查看守护时长，右键一键退出。

---

## 🚀 快速上手与使用

### 1. 命令行参数

```bash
# 启动常亮守护并常驻右下角托盘
keep-awake

# 查询当前是否处于常亮运行状态
keep-awake --status

# 向后台运行中的常亮助手发送安全退出信号并恢复原有系统设置
keep-awake --stop
```

### 2. Linux 平台构建与安装

确保本地已安装 Rust (1.80+)：

```bash
git clone https://github.com/LeonHerben/keep-awake.git
cd keep-awake
cargo build --release

# 安装到个人执行路径
cp target/release/keep-awake ~/.local/bin/
```

### 3. Windows 平台

直接在 [Releases](https://github.com/LeonHerben/keep-awake/releases) 页面下载预编译的 `keep-awake.exe` 单文件绿色程序，双击即可运行。

如需从源码编译：

```powershell
cargo build --release
```

---

## 📊 性能表现

| 评估指标 | 传统 Python/Electron 工具 | Keep-Awake (Rust) |
| :--- | :--- | :--- |
| **内存占用 (RSS)** | 70 ~ 200 MB | **~7 MB** |
| **冷启动耗时** | 300 ~ 1200 ms | **< 5 ms** |
| **外部运行时依赖** | Python / Node.js / GTK | **单一独立可执行文件** |
| **休眠锁可靠性** | 容易因异常退出残留 | **RAII + 内核自动解绑** |

---

## 📄 许可证

本项目采用 [MIT License](LICENSE) 开源协议。
