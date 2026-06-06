# 設定模組（config.rs）

## 資料結構

```rust
pub struct PaneConfig {
    pub enabled: bool,    // 是否啟用此格
    pub label: String,    // 視窗顯示名稱
    pub distro: String,   // WSL distro 名稱；空字串 = 使用系統預設
    pub command: String,  // 啟動後執行的指令；空字串 = 直接開 shell
}

pub struct AppConfig {
    pub panes: [PaneConfig; 4],  // 固定 4 格，對應 2×2 排列
}
```

## 設定檔

- 格式：JSON（serde_json pretty-print）
- 位置：`%APPDATA%\open-terminal\config.json`
- 自動建立父目錄，讀取失敗時回傳 `AppConfig::default()`

### 範例 config.json

```json
{
  "panes": [
    { "enabled": true,  "label": "Dev",   "distro": "Ubuntu-22.04", "command": "cd ~/project && bash" },
    { "enabled": true,  "label": "Git",   "distro": "Ubuntu-22.04", "command": "" },
    { "enabled": true,  "label": "Logs",  "distro": "",             "command": "tail -f /var/log/syslog" },
    { "enabled": false, "label": "Spare", "distro": "",             "command": "" }
  ]
}
```

## WSL Distro 掃描

`list_wsl_distros()` 呼叫 `wsl.exe --list --quiet`。

注意：wsl.exe 輸出為 **UTF-16LE**（含或不含 BOM），需手動解碼：

```rust
fn decode_utf16le(bytes: &[u8]) -> String {
    let words: Vec<u16> = bytes.chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    String::from_utf16_lossy(&words).to_owned()
}
```

掃描結果快取在 `App.distros: Vec<String>`，UI 右上角「↻ 重新整理」按鈕可重新掃描。

## 新增欄位注意事項

新增 `PaneConfig` 欄位時：
1. 加上 `#[serde(default)]` 或在 `Default` impl 給預設值，確保舊 config.json 仍可讀取。
2. 同步更新 `app.rs` 的 UI 邏輯。
