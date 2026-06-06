# open-terminal — 專案概覽

## 用途

一個 Windows GUI 工具（Rust + egui），一鍵開啟最多 4 個 Windows Terminal 視窗，
自動排列成 2×2 格局佔滿螢幕。每個格子可獨立設定：

- 視窗名稱（label）
- 要使用的 WSL distro（下拉選單，動態讀取）
- 啟動後執行的指令（command，留空直接開 WSL shell）
- 是否啟用（enabled toggle）

## 技術棧

| 項目 | 版本 |
|------|------|
| Rust edition | 2021 |
| egui / eframe | 0.28 |
| serde / serde_json | 1.x |
| windows crate | 0.58（Win32 API） |
| 編譯目標 | x86_64-pc-windows-gnu |

## 目錄結構

```
open-terminal/
├── src/
│   ├── main.rs       # 進入點、視窗設定、字型載入
│   ├── app.rs        # egui UI 邏輯
│   ├── config.rs     # 設定讀寫、WSL distro 掃描
│   └── launcher.rs   # 啟動 wt.exe 並排列視窗
├── docs/
│   └── skills/       # 本文件目錄
├── .claude/
│   └── skills/run/   # Claude run skill
└── Cargo.toml
```

## 設定檔位置

Windows：`%APPDATA%\open-terminal\config.json`

## 編譯 & 部署

```powershell
# 在 WSL 內執行
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/open-terminal.exe \
   /mnt/e/1_tools/05_windows-exe/02_open-terminal/
```

詳見 [build.md](build.md)。
