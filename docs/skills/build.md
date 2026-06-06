# 編譯與部署

## 前置需求

```bash
rustup target add x86_64-pc-windows-gnu
sudo apt install gcc-mingw-w64-x86-64   # Ubuntu/Debian
```

## 編譯指令

```bash
# Release 版（啟用 opt-level=3 + strip）
cargo build --release --target x86_64-pc-windows-gnu
```

產出位置：`target/x86_64-pc-windows-gnu/release/open-terminal.exe`

## 部署目標

```
E:\1_tools\05_windows-exe\02_open-terminal\open-terminal.exe
# WSL 路徑：/mnt/e/1_tools/05_windows-exe/02_open-terminal/
```

## 一鍵編譯 + 部署

```bash
cargo build --release --target x86_64-pc-windows-gnu && \
cp target/x86_64-pc-windows-gnu/release/open-terminal.exe \
   "/mnt/e/1_tools/05_windows-exe/02_open-terminal/"
```

## Cargo.toml profile

```toml
[profile.release]
opt-level = 3
strip = true
```

`strip = true` 會移除 debug symbols，大幅縮小 exe 體積。

## 常見問題

| 問題 | 原因 | 解法 |
|------|------|------|
| `linker not found` | 缺少 mingw | `sudo apt install gcc-mingw-w64-x86-64` |
| 中文亂碼 | 字型未載入 | 確認 `C:\Windows\Fonts\NotoSansTC-VF.ttf` 存在 |
| `wsl.exe not found` | 非 Windows 環境 | exe 只支援 Windows，請在 Windows 上執行 |
