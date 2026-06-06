use crate::config::AppConfig;
#[cfg(target_os = "windows")]
use crate::config::PaneConfig;

#[cfg(target_os = "windows")]
use std::collections::HashSet;

#[cfg(target_os = "windows")]
use std::time::Duration;

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::{BOOL, HWND, LPARAM, RECT},
    Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS},
    Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowRect, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible,
        SetWindowPos, ShowWindow, SystemParametersInfoW, HWND_TOP, SPI_GETWORKAREA,
        SWP_NOZORDER, SWP_SHOWWINDOW, SW_RESTORE, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    },
};

#[cfg(target_os = "windows")]
use windows::{
    Win32::Foundation::LPARAM as MON_LPARAM,
    Win32::Graphics::Gdi::{
        EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO,
    },
};

#[cfg(target_os = "windows")]
use crate::config::list_terminal_profiles;

pub struct LaunchError(pub String);

impl std::fmt::Display for LaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub fn launch(cfg: &AppConfig) -> Result<(), LaunchError> {
    #[cfg(not(target_os = "windows"))]
    {
        let _ = cfg;
        return Err(LaunchError("僅支援 Windows 平台".into()));
    }

    #[cfg(target_os = "windows")]
    launch_windows(cfg)
}

#[cfg(target_os = "windows")]
fn launch_windows(cfg: &AppConfig) -> Result<(), LaunchError> {
    use std::thread;

    let profiles = list_terminal_profiles();
    let mut known_windows = current_window_handles();

    // 主螢幕：SPI_GETWORKAREA（扣工作列）
    let primary_work = primary_workarea();
    launch_panes_on_workarea(
        &cfg.panes,
        primary_work,
        &profiles,
        &cfg.default_profile,
        &mut known_windows,
    )?;

    if cfg.dual_monitor {
        let monitors = list_monitors();
        // 找第二顆螢幕（排除與主螢幕工作區完全相同的）
        let second = monitors
            .into_iter()
            .find(|m| m.work.left != primary_work.left || m.work.top != primary_work.top);

        if let Some(mon) = second {
            // 短暫停讓主螢幕視窗穩定
            thread::sleep(Duration::from_millis(300));
            launch_panes_on_workarea(
                &cfg.panes2,
                mon.work,
                &profiles,
                &cfg.default_profile,
                &mut known_windows,
            )?;
        } else {
            return Err(LaunchError("找不到第二個螢幕".into()));
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn launch_panes_on_workarea(
    panes: &[PaneConfig; 4],
    work: RECT,
    profiles: &[crate::config::TerminalProfile],
    default_profile: &str,
    known_windows: &mut HashSet<usize>,
) -> Result<(), LaunchError> {
    use std::process::Command;
    use std::thread;

    let wa_x = work.left;
    let wa_y = work.top;
    let wa_w = work.right - work.left;
    let wa_h = work.bottom - work.top;
    let mid_x = wa_x + wa_w / 2;
    let mid_y = wa_y + wa_h / 2;

    let positions = [
        (wa_x,  wa_y,  mid_x - wa_x,       mid_y - wa_y),
        (mid_x, wa_y,  wa_x + wa_w - mid_x, mid_y - wa_y),
        (wa_x,  mid_y, mid_x - wa_x,       wa_y + wa_h - mid_y),
        (mid_x, mid_y, wa_x + wa_w - mid_x, wa_y + wa_h - mid_y),
    ];

    let mut launched: Vec<(HWND, usize)> = Vec::new();

    for (i, pane) in panes.iter().enumerate() {
        if !pane.enabled {
            continue;
        }

        let mut cmd = Command::new("wt.exe");
        cmd.args(["-w", "new"]);

        let profile_key = if pane.profile.trim().is_empty() {
            default_profile.trim()
        } else {
            pane.profile.trim()
        };
        let profile_name = profiles
            .iter()
            .find(|p| p.guid == profile_key || p.name == profile_key)
            .map(|p| p.name.as_str());

        if !pane.command.trim().is_empty() {
            if let Some(pn) = profile_name {
                cmd.args(["-p", pn, "wsl.exe", "--", "bash", "-lc", &pane.command]);
            } else if profile_key.is_empty() {
                cmd.args(["wsl.exe", "--", "bash", "-lc", &pane.command]);
            } else {
                cmd.args(["wsl.exe", "-d", profile_key, "--", "bash", "-lc", &pane.command]);
            }
        } else if let Some(pn) = profile_name {
            cmd.args(["-p", pn]);
        } else if profile_key.is_empty() {
            cmd.args(["wsl.exe"]);
        } else {
            cmd.args(["wsl.exe", "-d", profile_key]);
        }

        cmd.spawn()
            .map_err(|e| LaunchError(format!("無法啟動 wt.exe: {e}")))?;

        let hwnd = wait_for_new_window(known_windows, Duration::from_secs(5))
            .ok_or_else(|| LaunchError("找不到新開啟的 Windows Terminal 視窗".into()))?;
        known_windows.insert(hwnd.0 as usize);
        launched.push((hwnd, i));

        thread::sleep(Duration::from_millis(300));
    }

    for (hwnd, pos_idx) in &launched {
        let (x, y, w, h) = positions[*pos_idx];
        snap_window(*hwnd, x, y, w, h);
    }

    Ok(())
}

// ── 螢幕工作區 ───────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn primary_workarea() -> RECT {
    unsafe {
        let mut r = RECT::default();
        let _ = SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            Some(&mut r as *mut _ as *mut _),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        );
        r
    }
}

#[cfg(target_os = "windows")]
struct MonitorInfo {
    work: RECT,
}

#[cfg(target_os = "windows")]
fn list_monitors() -> Vec<MonitorInfo> {
    struct Data {
        monitors: Vec<MonitorInfo>,
    }

    extern "system" fn callback(
        hmon: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        lparam: MON_LPARAM,
    ) -> BOOL {
        let data = unsafe { &mut *(lparam.0 as *mut Data) };
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if unsafe { GetMonitorInfoW(hmon, &mut info).as_bool() } {
            data.monitors.push(MonitorInfo { work: info.rcWork });
        }
        BOOL(1)
    }

    let mut data = Data { monitors: Vec::new() };
    unsafe {
        let _ = EnumDisplayMonitors(
            HDC::default(),
            None,
            Some(callback),
            MON_LPARAM(&mut data as *mut _ as isize),
        );
    }
    data.monitors
}

// ── 視窗列舉 ─────────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn current_window_handles() -> HashSet<usize> {
    enumerate_windows()
        .into_iter()
        .map(|hwnd| hwnd.0 as usize)
        .collect()
}

#[cfg(target_os = "windows")]
fn enumerate_windows() -> Vec<HWND> {
    struct SearchData {
        windows: Vec<HWND>,
    }

    extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = unsafe { &mut *(lparam.0 as *mut SearchData) };
        if is_candidate_window(hwnd) {
            data.windows.push(hwnd);
        }
        BOOL(1)
    }

    let mut data = SearchData { windows: Vec::new() };
    unsafe {
        let _ = EnumWindows(Some(enum_callback), LPARAM(&mut data as *mut _ as isize));
    }
    data.windows
}

#[cfg(target_os = "windows")]
fn wait_for_new_window(known_windows: &HashSet<usize>, timeout: Duration) -> Option<HWND> {
    use std::thread;
    use std::time::Instant;

    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Some(hwnd) = enumerate_windows()
            .into_iter()
            .find(|hwnd| !known_windows.contains(&(hwnd.0 as usize)))
        {
            return Some(hwnd);
        }
        thread::sleep(Duration::from_millis(100));
    }
    None
}

#[cfg(target_os = "windows")]
fn is_candidate_window(hwnd: HWND) -> bool {
    if !unsafe { IsWindowVisible(hwnd).as_bool() } {
        return false;
    }
    let title_len = unsafe { GetWindowTextLengthW(hwnd) };
    if title_len <= 0 {
        return false;
    }
    let mut title = vec![0u16; title_len as usize + 1];
    let copied = unsafe { GetWindowTextW(hwnd, &mut title) };
    if copied <= 0 {
        return false;
    }
    let title = String::from_utf16_lossy(&title[..copied as usize]);
    !title.trim().is_empty()
}

// ── DWM 邊框補償 ─────────────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn dwm_border(hwnd: HWND) -> (i32, i32, i32, i32) {
    unsafe {
        let mut frame = RECT::default();
        let mut win = RECT::default();
        let ok = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut frame as *mut _ as *mut _,
            std::mem::size_of::<RECT>() as u32,
        )
        .is_ok();
        let _ = GetWindowRect(hwnd, &mut win);
        if ok && (win.right > win.left) {
            (
                frame.left - win.left,
                frame.top - win.top,
                win.right - frame.right,
                win.bottom - frame.bottom,
            )
        } else {
            (0, 0, 0, 0)
        }
    }
}

#[cfg(target_os = "windows")]
fn snap_window(hwnd: HWND, target_x: i32, target_y: i32, target_w: i32, target_h: i32) {
    unsafe {
        let _ = ShowWindow(hwnd, SW_RESTORE);

        let (bl, bt, br, bb) = dwm_border(hwnd);
        let _ = SetWindowPos(
            hwnd, HWND_TOP,
            target_x - bl, target_y - bt,
            target_w + bl + br, target_h + bt + bb,
            SWP_NOZORDER | SWP_SHOWWINDOW,
        );

        // 貼近螢幕邊緣後邊框縮為 0，第二次修正
        let (bl2, bt2, br2, bb2) = dwm_border(hwnd);
        let _ = SetWindowPos(
            hwnd, HWND_TOP,
            target_x - bl2, target_y - bt2,
            target_w + bl2 + br2, target_h + bt2 + bb2,
            SWP_NOZORDER | SWP_SHOWWINDOW,
        );
    }
}
