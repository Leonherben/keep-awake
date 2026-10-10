use std::process::Command;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;
use tray_item::{IconSource, TrayItem};
use windows_sys::Win32::UI::WindowsAndMessaging::{LoadIconW, IDI_APPLICATION};

pub struct TrayHandleWrapper {
    _tray: Option<TrayItem>,
}

impl TrayHandleWrapper {
    pub async fn update(&self) {
        // Windows 托盘状态刷新
    }
}

pub fn send_notification(summary: &str, body: &str, _icon: &str) {
    let script = format!(
        "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null; \
         $template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
         $textNodes = $template.GetElementsByTagName('text'); \
         $textNodes.Item(0).AppendChild($template.CreateTextNode('{}')) | Out-Null; \
         $textNodes.Item(1).AppendChild($template.CreateTextNode('{}')) | Out-Null; \
         $notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('KeepAwake'); \
         $notification = [Windows.UI.Notifications.ToastNotification]::new($template); \
         $notifier.Show($notification);",
        summary.replace('\'', "''"),
        body.replace('\'', "''")
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", &script])
        .spawn();
}

pub async fn spawn_tray(
    start_time: Instant,
    quit_tx: UnboundedSender<()>,
) -> Result<TrayHandleWrapper, Box<dyn std::error::Error + Send + Sync>> {
    // 优先级 1: 尝试由 winres 嵌入的 PE 资源图标 ("app-icon")
    // 优先级 2: 尝试资源 ID 1
    // 优先级 3: 兜底使用 Windows 系统默认应用图标 (IDI_APPLICATION)，确保 100% 成功显现
    let mut tray = TrayItem::new("保持常亮助手", IconSource::Resource("app-icon"))
        .or_else(|_| TrayItem::new("保持常亮助手", IconSource::Resource("1")))
        .or_else(|_| {
            let sys_icon = unsafe { LoadIconW(0, IDI_APPLICATION) };
            TrayItem::new("保持常亮助手", IconSource::RawIcon(sys_icon as _))
        })?;

    let _ = tray.add_label("🟢 保持常亮运行中");

    let start_clone = start_time;
    let _ = tray.add_menu_item("ℹ 查看运行状态", move || {
        let elapsed = start_clone.elapsed().as_secs();
        let hrs = elapsed / 3600;
        let mins = (elapsed % 3600) / 60;
        let secs = elapsed % 60;
        send_notification(
            "保持常亮运行中",
            &format!(
                "已持续守护: {:02}小时 {:02}分 {:02}秒\n屏幕保持常亮，阻止休眠。",
                hrs, mins, secs
            ),
            "",
        );
    });

    let _ = tray.add_menu_item("⏹ 恢复原样并退出", move || {
        let _ = quit_tx.send(());
    });

    Ok(TrayHandleWrapper { _tray: Some(tray) })
}
