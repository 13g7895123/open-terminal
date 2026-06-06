# Launcher 模組（launcher.rs）

## 功能

呼叫 `wt.exe`（Windows Terminal）開啟新視窗，並透過 Win32 API 將視窗排列到 2×2 格局。

## 啟動邏輯

```
對每個 enabled 的 pane：
  wt.exe -w new wsl.exe [-d <distro>] [-- bash -c <command>]
  sleep 800ms   ← 避免視窗競爭
全部啟動後 sleep 1500ms
對每個 pid 找 HWND → SetWindowPos 到對應象限
```

## wt.exe 指令組合

| distro | command | 指令 |
|--------|---------|------|
| 空 | 空 | `wt.exe -w new wsl.exe` |
| 有 | 空 | `wt.exe -w new wsl.exe -d <distro>` |
| 空 | 有 | `wt.exe -w new wsl.exe -- bash -c <cmd>` |
| 有 | 有 | `wt.exe -w new wsl.exe -d <distro> -- bash -c <cmd>` |

## 視窗排列

螢幕解析度透過 `GetSystemMetrics(SM_CXSCREEN / SM_CYSCREEN)` 動態取得。

```
位置索引對應：
  0 = 左上 (0,       0,      w/2, h/2)
  1 = 右上 (w/2,     0,      w/2, h/2)
  2 = 左下 (0,       h/2,    w/2, h/2)
  3 = 右下 (w/2,     h/2,    w/2, h/2)
```

disabled 的 pane 不啟動，其對應的螢幕位置留空（不影響其他視窗排列位置）。

## Win32 API 使用

```rust
// 透過 PID 找 HWND
EnumWindows(callback, lparam)
GetWindowThreadProcessId(hwnd, &mut pid)

// 排列視窗
ShowWindow(hwnd, SW_RESTORE)       // 先還原最小化狀態
SetWindowPos(hwnd, HWND_TOP, x, y, w, h, SWP_NOZORDER | SWP_SHOWWINDOW)
```

## 非 Windows 平台

`#[cfg(not(target_os = "windows"))]` 分支直接回傳錯誤，不做任何事。
