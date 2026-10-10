fn main() {
    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        res.set("FileDescription", "Keep-Awake Daemon");
        res.set("ProductName", "Keep-Awake");
        let _ = res.compile();
    }
}
