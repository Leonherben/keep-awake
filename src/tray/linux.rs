use ksni::menu::{MenuItem, StandardItem};
use ksni::{ToolTip, Tray, TrayMethods};
use std::process::Command;
use std::time::Instant;
use tokio::sync::mpsc::UnboundedSender;

pub struct KeepAwakeTray {
    start_time: Instant,
    quit_tx: UnboundedSender<()>,
}

impl KeepAwakeTray {
    pub fn new(start_time: Instant, quit_tx: UnboundedSender<()>) -> Self {
        Self {
            start_time,
            quit_tx,
        }
    }

    pub fn show_status_notification(&self) {
        let elapsed = self.start_time.elapsed().as_secs();
        let hrs = elapsed / 3600;
        let mins = (elapsed % 3600) / 60;
        let secs = elapsed % 60;

        send_notification(
            "保持常亮运行中",
            &format!(
                "已持续守护: {:02}小时 {:02}分 {:02}秒\n屏幕保持常亮，阻止熄屏与休眠保护。",
                hrs, mins, secs
            ),
            "display-brightness-symbolic",
        );
    }
}

pub fn send_notification(summary: &str, body: &str, icon: &str) {
    let _ = Command::new("notify-send")
        .args(["-a", "保持常亮助手", "-i", icon, summary, body])
        .spawn();
}

impl Tray for KeepAwakeTray {
    fn id(&self) -> String {
        "org.malus.keepawake".into()
    }

    fn title(&self) -> String {
        "保持常亮助手".into()
    }

    fn icon_name(&self) -> String {
        "display-brightness-symbolic".into()
    }

    fn tool_tip(&self) -> ToolTip {
        let elapsed = self.start_time.elapsed().as_secs();
        let hrs = elapsed / 3600;
        let mins = (elapsed % 3600) / 60;
        let secs = elapsed % 60;

        ToolTip {
            title: "保持常亮助手".into(),
            description: format!(
                "常亮运行中: {:02}:{:02}:{:02}\n阻止熄屏与系统休眠",
                hrs, mins, secs
            ),
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        self.show_status_notification();
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let quit_tx = self.quit_tx.clone();

        vec![
            StandardItem {
                label: "ℹ 查看当前守护状态".into(),
                activate: Box::new(|this: &mut Self| {
                    this.show_status_notification();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "⏹ 恢复原样并退出".into(),
                activate: Box::new(move |_this: &mut Self| {
                    let _ = quit_tx.send(());
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub struct TrayHandleWrapper {
    inner: Option<ksni::Handle<KeepAwakeTray>>,
}

impl TrayHandleWrapper {
    pub async fn update(&self) {
        if let Some(ref h) = self.inner {
            h.update(|_t| {}).await;
        }
    }
}

pub async fn spawn_tray(start_time: Instant, quit_tx: UnboundedSender<()>) -> Result<TrayHandleWrapper, Box<dyn std::error::Error + Send + Sync>> {
    let tray = KeepAwakeTray::new(start_time, quit_tx);
    let handle = tray.spawn().await?;
    Ok(TrayHandleWrapper { inner: Some(handle) })
}
