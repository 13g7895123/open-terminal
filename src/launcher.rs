use crate::config::PaneConfig;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::{HWND, BOOL, LPARAM},
    Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowThreadProcessId, SetWindowPos,
        ShowWindow, GetSystemMetrics,
        SWP_NOZORDER, SWP_SHOWWINDOW,
        SW_RESTORE,
        SM_CXSCREEN, SM_CYSCREEN,
        HWND_TOP,
    },
};

pub struct LaunchError(pub String);

impl std::fmt::Display for LaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn launch(panes: &[PaneConfig; 4]) -> Result<(), LaunchError> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = panes;
        return Err(LaunchError("僅支援 Windows 平台".into()));
    }

    #[cfg(target_os = "windows")]
    launch_windows(panes)
}

#[cfg(target_os = "windows")]
fn launch_windows(panes: &[PaneConfig; 4]) -> Result<(), LaunchError> {
    use std::process::Command;
    use std::time::Duration;
    use std::thread;
    let screen_w = unsafe { GetSystemMetrics(SM_CXSCREEN) };
    let screen_h = unsafe { GetSystemMetrics(SM_CYSCREEN) };
    let half_w = screen_w / 2;
    let half_h = screen_h / 2;

    // 2x2 grid positions: (x, y, w, h)
    let positions = [
        (0,       0,       half_w, half_h),
        (half_w,  0,       half_w, half_h),
        (0,       half_h,  half_w, half_h),
        (half_w,  half_h,  half_w, half_h),
    ];

    let mut pids: Vec<u32> = Vec::new();

    for pane in panes.iter() {
        let mut cmd = Command::new("wt.exe");
        cmd.args(["-w", "new"]);
        if !pane.command.trim().is_empty() {
            cmd.args(["wsl.exe", "--", "bash", "-c", &pane.command]);
        } else {
            cmd.args(["wsl.exe"]);
        }

        let child = cmd.spawn().map_err(|e| LaunchError(format!("無法啟動 wt.exe: {e}")))?;
        pids.push(child.id());

        // 等待視窗出現，避免開太快
        thread::sleep(Duration::from_millis(800));
    }

    // 等待所有視窗完全載入
    thread::sleep(Duration::from_millis(1500));

    // 對每個 PID 找到對應的頂層視窗並排列
    for (i, &pid) in pids.iter().enumerate() {
        if let Some(hwnd) = find_window_by_pid(pid) {
            let (x, y, w, h) = positions[i];
            unsafe {
                ShowWindow(hwnd, SW_RESTORE);
                let _ = SetWindowPos(
                    hwnd,
                    HWND_TOP,
                    x, y, w, h,
                    SWP_NOZORDER | SWP_SHOWWINDOW,
                );
            }
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn find_window_by_pid(target_pid: u32) -> Option<HWND> {
    struct SearchData {
        target_pid: u32,
        found: Option<HWND>,
    }

    extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = unsafe { &mut *(lparam.0 as *mut SearchData) };
        let mut pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if pid == data.target_pid {
            data.found = Some(hwnd);
            return BOOL(0); // 停止列舉
        }
        BOOL(1)
    }

    let mut data = SearchData { target_pid, found: None };
    unsafe {
        let _ = EnumWindows(
            Some(enum_callback),
            LPARAM(&mut data as *mut _ as isize),
        );
    }
    data.found
}
