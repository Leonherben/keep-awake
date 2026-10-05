mod inhibitor;
mod tray;

use clap::Parser;
use inhibitor::InhibitorManager;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tray::{send_notification, spawn_tray};

#[derive(Parser, Debug)]
#[command(name = "keep-awake", about = "跨平台屏幕常亮与休眠保护守护助手 (Linux & Windows)")]
struct Args {
    /// 启动并保留在托盘
    #[arg(long, default_value_t = true)]
    tray: bool,

    /// 停止常亮并退出已在运行的实例
    #[arg(long)]
    stop: bool,

    /// 查询当前运行状态
    #[arg(long)]
    status: bool,
}

fn get_pid_file_path() -> PathBuf {
    #[cfg(unix)]
    {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            PathBuf::from(runtime_dir).join("keep-awake.pid")
        } else {
            std::env::temp_dir().join(format!("keep-awake-{}.pid", whoami_user()))
        }
    }
    #[cfg(windows)]
    {
        if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
            let dir = PathBuf::from(local_appdata).join("keep-awake");
            let _ = fs::create_dir_all(&dir);
            dir.join("keep-awake.pid")
        } else {
            std::env::temp_dir().join("keep-awake.pid")
        }
    }
}

#[cfg(unix)]
fn whoami_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "user".into())
}

fn is_pid_running(pid: u32) -> bool {
    #[cfg(unix)]
    {
        Command::new("kill")
            .args(["-0", &pid.to_string()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    #[cfg(windows)]
    {
        Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains(&pid.to_string()))
            .unwrap_or(false)
    }
}

fn stop_existing_instance() -> bool {
    let pid_file = get_pid_file_path();
    if let Ok(content) = fs::read_to_string(&pid_file) {
        if let Ok(pid) = content.trim().parse::<u32>() {
            if is_pid_running(pid) {
                #[cfg(unix)]
                let _ = Command::new("kill")
                    .args(["-TERM", &pid.to_string()])
                    .status();

                #[cfg(windows)]
                let _ = Command::new("taskkill")
                    .args(["/PID", &pid.to_string(), "/F"])
                    .status();

                println!("已向运行中的常亮助手 (PID: {}) 发送退出信号。", pid);
                return true;
            }
        }
    }

    #[cfg(unix)]
    {
        let pgrep = Command::new("pgrep").args(["-x", "keep-awake"]).output();
        if let Ok(out) = pgrep {
            let current_pid = std::process::id();
            let pids: Vec<String> = String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|s| s.trim().to_string())
                .filter(|p| !p.is_empty() && p != &current_pid.to_string())
                .collect();

            if !pids.is_empty() {
                for p in &pids {
                    let _ = Command::new("kill").args(["-TERM", p]).status();
                }
                println!("已终止运行中的常亮进程 (PID: {})。", pids.join(", "));
                return true;
            }
        }
    }

    println!("未发现正在运行的常亮守护实例。");
    false
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.stop {
        stop_existing_instance();
        return Ok(());
    }

    if args.status {
        let pid_file = get_pid_file_path();
        if let Ok(content) = fs::read_to_string(&pid_file) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                if is_pid_running(pid) {
                    println!("🟢 keep-awake 常亮守护正在运行中 (PID: {})", pid);
                    return Ok(());
                }
            }
        }
        println!("⚪ keep-awake 当前未运行。");
        return Ok(());
    }

    // 单例检查
    let pid_file = get_pid_file_path();
    if let Ok(content) = fs::read_to_string(&pid_file) {
        if let Ok(pid) = content.trim().parse::<u32>() {
            if is_pid_running(pid) {
                println!(
                    "常亮助手已在运行中 (PID: {})。如需退出请执行 keep-awake --stop",
                    pid
                );
                return Ok(());
            }
        }
    }

    // 记录当前 PID
    let current_pid = std::process::id();
    let _ = fs::write(&pid_file, current_pid.to_string());

    println!("🚀 保持常亮守护助手启动中 [PID: {}]...", current_pid);

    // 1. 初始化并激活底层系统防休眠抑制锁
    let mut inhibitor = InhibitorManager::new().await;
    if inhibitor.is_inhibited() {
        println!("✅ 系统休眠与熄屏抑制锁已激活。");
    } else {
        eprintln!("⚠️ 提示: 正在运行于后台常亮模式。");
    }

    // 发送启动提示通知
    send_notification(
        "保持常亮已激活",
        "屏幕将保持唤醒，阻止休眠；退出后立即恢复原样。",
        "display-brightness-symbolic",
    );

    // 2. 启动系统托盘
    let (quit_tx, mut quit_rx) = mpsc::unbounded_channel();
    let start_time = Instant::now();

    let tray_handle = match spawn_tray(start_time, quit_tx).await {
        Ok(handle) => {
            println!("✅ 系统托盘指示器创建成功。");
            Some(handle)
        }
        Err(e) => {
            eprintln!("⚠️ 托盘创建警告: {} (仍将在后台保持常亮)", e);
            None
        }
    };

    // 3. 监听退出信号
    #[cfg(unix)]
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;

    // 定时器循环
    let mut idle_ticker = interval(Duration::from_secs(60));
    let mut tray_ticker = interval(Duration::from_secs(5));

    println!("✨ 保持常亮运行中，可通过右下角托盘图标查看状态或退出。");

    loop {
        #[cfg(unix)]
        {
            tokio::select! {
                _ = quit_rx.recv() => {
                    println!("收到来自托盘菜单的退出请求。");
                    break;
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("收到 Ctrl+C (SIGINT) 中断信号。");
                    break;
                }
                _ = sigterm.recv() => {
                    println!("收到 SIGTERM 终止信号。");
                    break;
                }
                _ = idle_ticker.tick() => {
                    InhibitorManager::reset_idle();
                }
                _ = tray_ticker.tick() => {
                    if let Some(ref handle) = tray_handle {
                        handle.update().await;
                    }
                }
            }
        }
        #[cfg(not(unix))]
        {
            tokio::select! {
                _ = quit_rx.recv() => {
                    println!("收到来自托盘菜单的退出请求。");
                    break;
                }
                _ = tokio::signal::ctrl_c() => {
                    println!("收到 Ctrl+C 中断信号。");
                    break;
                }
                _ = idle_ticker.tick() => {
                    InhibitorManager::reset_idle();
                }
                _ = tray_ticker.tick() => {
                    if let Some(ref handle) = tray_handle {
                        handle.update().await;
                    }
                }
            }
        }
    }

    // 4. 清理并退出
    println!("正在释放系统抑制锁并恢复原有设置...");
    inhibitor.uninhibit().await;
    let _ = fs::remove_file(&pid_file);

    send_notification(
        "已恢复系统默认设置",
        "常亮模式已结束，屏幕熄屏与休眠保护已重新生效。",
        "preferences-system-power",
    );

    println!("👋 已退出，系统设置已完全恢复。");
    Ok(())
}
