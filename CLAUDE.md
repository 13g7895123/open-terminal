# open-terminal — Claude 指引

## 必讀文件

開始任何任務前，請先閱讀以下 docs/skills 文件：

| 文件 | 內容 |
|------|------|
| [docs/skills/overview.md](docs/skills/overview.md) | 專案概覽、技術棧、目錄結構 |
| [docs/skills/build.md](docs/skills/build.md) | 編譯指令、部署路徑、常見問題 |
| [docs/skills/config.md](docs/skills/config.md) | 設定資料結構、WSL distro 掃描邏輯 |
| [docs/skills/ui.md](docs/skills/ui.md) | egui UI 架構、調色盤、版面、已知 API 陷阱 |
| [docs/skills/launcher.md](docs/skills/launcher.md) | wt.exe 啟動邏輯、Win32 視窗排列 |

## 編譯 & 部署規則

**每次修改完成後，先編譯，複製前必須詢問目前環境（公司或家裡）：**

```bash
cargo build --release --target x86_64-pc-windows-gnu
```

部署路徑依環境而定：

| 環境 | 路徑 |
|------|------|
| 公司（C 槽） | `/mnt/c/Jarvis/20_tools/7_windows-exe/01_open-terminal/` |
| 家裡（E 槽） | `/mnt/e/1_tools/05_windows-exe/02_open-terminal/` |

複製指令範例（確認環境後再執行）：
```bash
# 公司
cp target/x86_64-pc-windows-gnu/release/open-terminal.exe \
   "/mnt/c/Jarvis/20_tools/7_windows-exe/01_open-terminal/"

# 家裡
cp target/x86_64-pc-windows-gnu/release/open-terminal.exe \
   "/mnt/e/1_tools/05_windows-exe/02_open-terminal/"
```

- 永遠使用 `--target x86_64-pc-windows-gnu`
- **複製前必須先詢問使用者目前是在公司還是家裡**
- 不需要 commit，DO NOT auto commit

## egui 0.28 已知 API 差異

- `Frame::none()` — 不是 `Frame::new()`
- `ComboBox::from_id_source()` — 不是 `from_id_salt()`
- `FontData::from_owned(data).into()` — 轉成 `Arc<FontData>`

## 語言

- 回覆使用繁體中文（zh-tw）
