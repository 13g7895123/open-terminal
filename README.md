# open-terminal

一個 Windows 桌面工具，一鍵開啟四個 WSL 終端機視窗，並自動排列成 2×2 格局佔滿螢幕。

## 功能

- 以 2×2 網格排列四個 Windows Terminal（`wt.exe`）視窗
- 每個視窗可設定自訂名稱與啟動指令
- 設定自動儲存至 `%APPDATA%\open-terminal\config.json`，下次開啟自動載入
- 留空指令則直接開啟 WSL shell

## 畫面預覽

```
┌─────────────┬─────────────┐
│  Terminal 1 │  Terminal 2 │
├─────────────┼─────────────┤
│  Terminal 3 │  Terminal 4 │
└─────────────┴─────────────┘
```

## 環境需求

- Windows 10 / 11
- [Windows Terminal](https://aka.ms/terminal)（`wt.exe`）
- WSL 已安裝並設定完成
- Rust 工具鏈（僅編譯時需要）

## 編譯與執行

```bash
# 編譯 release 版本
cargo build --release

# 直接執行
cargo run
```

產出的執行檔位於 `target/release/open-terminal.exe`。

## 使用方式

1. 啟動程式
2. 在四個格子中填入視窗名稱與要執行的指令（可留空）
3. 點擊「🚀 啟動四個終端機」
4. 四個 WSL 視窗將自動開啟並排列至螢幕四個角落

## 技術架構

| 模組 | 說明 |
|------|------|
| `main.rs` | 程式進入點，初始化 egui 視窗 |
| `app.rs` | GUI 介面與使用者互動邏輯 |
| `config.rs` | 設定讀寫（JSON 格式） |
| `launcher.rs` | 啟動 `wt.exe` 並透過 Win32 API 排列視窗位置 |

## 依賴套件

- [eframe / egui](https://github.com/emilk/egui) — 跨平台 GUI 框架
- [serde / serde_json](https://serde.rs/) — 設定序列化
- [windows-rs](https://github.com/microsoft/windows-rs) — Win32 API 綁定
