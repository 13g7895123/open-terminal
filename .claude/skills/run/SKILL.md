---
description: 編譯 open-terminal 並部署到 E:\1_tools\05_windows-exe\02_open-terminal\
---

# Run Skill — open-terminal

## 部署目標

```
/mnt/e/1_tools/05_windows-exe/02_open-terminal/open-terminal.exe
```

## 步驟

1. 編譯 release 版（Windows GNU target）
2. 確認產出檔案存在
3. 複製到部署目標

```bash
cargo build --release --target x86_64-pc-windows-gnu && \
cp target/x86_64-pc-windows-gnu/release/open-terminal.exe \
   "/mnt/e/1_tools/05_windows-exe/02_open-terminal/" && \
echo "✓ 部署完成：/mnt/e/1_tools/05_windows-exe/02_open-terminal/open-terminal.exe" && \
ls -lh "/mnt/e/1_tools/05_windows-exe/02_open-terminal/open-terminal.exe"
```

## 注意事項

- 編譯目標為 `x86_64-pc-windows-gnu`，需要 `gcc-mingw-w64-x86-64`
- exe 執行時會從 `C:\Windows\Fonts\` 動態載入中文字型，不打包進 exe
- 詳細說明見 [docs/skills/build.md](../../../docs/skills/build.md)
