use std::os::fd::OwnedFd;
use std::process::Command;
use zbus::Connection;

pub struct InhibitorManager {
    session_conn: Option<Connection>,
    system_conn: Option<Connection>,
    session_cookie: Option<u32>,
    screensaver_cookie: Option<u32>,
    systemd_fd: Option<OwnedFd>,
}

impl InhibitorManager {
    pub async fn new() -> Self {
        let session_conn = Connection::session().await.ok();
        let system_conn = Connection::system().await.ok();

        let mut mgr = Self {
            session_conn,
            system_conn,
            session_cookie: None,
            screensaver_cookie: None,
            systemd_fd: None,
        };
        mgr.inhibit().await;
        mgr
    }

    pub async fn inhibit(&mut self) {
        // 1. org.gnome.SessionManager (桌面会话层: 阻止休眠与空闲熄屏, 4 | 8 = 12)
        if let Some(ref conn) = self.session_conn {
            if self.session_cookie.is_none() {
                match conn
                    .call_method(
                        Some("org.gnome.SessionManager"),
                        "/org/gnome/SessionManager",
                        Some("org.gnome.SessionManager"),
                        "Inhibit",
                        &("keep-awake", 0u32, "用户开启保持常亮（阻止熄屏与休眠）", 12u32),
                    )
                    .await
                {
                    Ok(reply) => {
                        self.session_cookie = reply.body().deserialize().ok();
                    }
                    Err(e) => eprintln!("SessionManager Inhibit 异常: {e}"),
                }
            }

            // 2. org.freedesktop.ScreenSaver (X11/桌面屏保与黑屏层)
            if self.screensaver_cookie.is_none() {
                match conn
                    .call_method(
                        Some("org.freedesktop.ScreenSaver"),
                        "/org/freedesktop/ScreenSaver",
                        Some("org.freedesktop.ScreenSaver"),
                        "Inhibit",
                        &("keep-awake", "用户开启保持常亮（阻止屏幕保护与黑屏）"),
                    )
                    .await
                {
                    Ok(reply) => {
                        self.screensaver_cookie = reply.body().deserialize().ok();
                    }
                    Err(e) => eprintln!("ScreenSaver Inhibit 异常: {e}"),
                }
            }
        }

        // 3. org.freedesktop.login1 (systemd 内核与电源管理器层: 阻止休眠与合盖挂起)
        if let Some(ref conn) = self.system_conn {
            if self.systemd_fd.is_none() {
                match conn
                    .call_method(
                        Some("org.freedesktop.login1"),
                        "/org/freedesktop/login1",
                        Some("org.freedesktop.login1.Manager"),
                        "Inhibit",
                        &(
                            "sleep:idle:handle-lid-switch",
                            "KeepAwake",
                            "用户开启常亮模式，保持后台任务不间断运行",
                            "block",
                        ),
                    )
                    .await
                {
                    Ok(reply) => {
                        let owned_fd: Option<zbus::zvariant::OwnedFd> =
                            reply.body().deserialize().ok();
                        if let Some(fd) = owned_fd {
                            self.systemd_fd = Some(fd.into());
                        }
                    }
                    Err(e) => eprintln!("systemd login1 Inhibit 异常: {e}"),
                }
            }
        }

        // 4. 重置一次 X11 计时器
        Self::reset_idle();
    }

    pub async fn uninhibit(&mut self) {
        if let Some(ref conn) = self.session_conn {
            if let Some(cookie) = self.session_cookie.take() {
                let _ = conn
                    .call_method(
                        Some("org.gnome.SessionManager"),
                        "/org/gnome/SessionManager",
                        Some("org.gnome.SessionManager"),
                        "Uninhibit",
                        &(cookie,),
                    )
                    .await;
            }

            if let Some(cookie) = self.screensaver_cookie.take() {
                let _ = conn
                    .call_method(
                        Some("org.freedesktop.ScreenSaver"),
                        "/org/freedesktop/ScreenSaver",
                        Some("org.freedesktop.ScreenSaver"),
                        "UnInhibit",
                        &(cookie,),
                    )
                    .await;
            }
        }

        self.systemd_fd = None;
    }

    pub fn is_inhibited(&self) -> bool {
        self.session_cookie.is_some()
            || self.screensaver_cookie.is_some()
            || self.systemd_fd.is_some()
    }

    pub fn reset_idle() {
        let _ = Command::new("xset")
            .args(["s", "reset"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
    }
}
