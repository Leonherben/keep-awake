#[link(name = "kernel32")]
unsafe extern "system" {
    fn SetThreadExecutionState(es_flags: u32) -> u32;
}

// Win32 API 常量定义
const ES_CONTINUOUS: u32 = 0x8000_0000;
const ES_SYSTEM_REQUIRED: u32 = 0x0000_0001;
const ES_DISPLAY_REQUIRED: u32 = 0x0000_0002;
const ES_AWAYMODE_REQUIRED: u32 = 0x0000_0040;

pub struct InhibitorManager {
    is_active: bool,
}

impl InhibitorManager {
    pub async fn new() -> Self {
        let mut mgr = Self { is_active: false };
        mgr.inhibit().await;
        mgr
    }

    pub async fn inhibit(&mut self) {
        unsafe {
            // 阻止系统待机、阻止屏幕变暗关闭，并开启 AwayMode
            let prev = SetThreadExecutionState(
                ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED | ES_AWAYMODE_REQUIRED,
            );
            self.is_active = prev != 0;
        }
    }

    pub async fn uninhibit(&mut self) {
        if self.is_active {
            unsafe {
                // 清除标志，系统恢复原有的休眠与熄屏计划
                SetThreadExecutionState(ES_CONTINUOUS);
            }
            self.is_active = false;
        }
    }

    pub fn is_inhibited(&self) -> bool {
        self.is_active
    }

    pub fn reset_idle() {
        unsafe {
            SetThreadExecutionState(
                ES_CONTINUOUS | ES_SYSTEM_REQUIRED | ES_DISPLAY_REQUIRED,
            );
        }
    }
}

impl Drop for InhibitorManager {
    fn drop(&mut self) {
        if self.is_active {
            unsafe {
                SetThreadExecutionState(ES_CONTINUOUS);
            }
        }
    }
}
