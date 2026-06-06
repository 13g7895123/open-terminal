# UI 模組（app.rs）

## 架構

`App` struct 實作 `eframe::App`，每幀呼叫 `update()`。

```rust
pub struct App {
    pub config: AppConfig,
    distros: Vec<String>,   // 快取 WSL distro 清單
    status: String,         // 底部狀態訊息
    status_ok: bool,        // true=綠色成功, false=紅色錯誤
    launching: bool,        // 啟動中 flag（避免重複點擊）
}
```

## 調色盤

| 常數 | Hex | 用途 |
|------|-----|------|
| `BG` | `#0F172A` | 背景 |
| `CARD` | `#1E293B` | 啟用卡片背景 |
| `CARD_DISABLED` | `#16201C` | 停用卡片背景 |
| `BORDER` | `#475569` | 卡片邊框 |
| `BORDER_FOCUS` | `#22C55E` | hover/focus 邊框 |
| `TEXT` | `#F8FAFC` | 主文字 |
| `TEXT_MUTED` | `#94A3B8` | 次要文字 |
| `ACCENT` | `#22C55E` | 綠色 CTA、accent |
| `BTN_SEC` | `#334155` | 次要按鈕背景 |
| `STATUS_ERR` | `#EF4444` | 錯誤狀態 |

## 版面結構

```
┌─────────────────────────────────────────┐
│ ● Open Terminal              ↻ 重新整理  │  ← 標題列
├─────────────────────────────────────────┤
│ ┌──────────────┐ ┌──────────────┐       │
│ │ ① 左上  啟用□│ │ ② 右上  啟用□│       │  ← 卡片列 1
│ │ 名稱 [____] │ │ 名稱 [____] │       │
│ │ Distro [▼] │ │ Distro [▼] │       │
│ │ 指令 [____] │ │ 指令 [____] │       │
│ └──────────────┘ └──────────────┘       │
│ ┌──────────────┐ ┌──────────────┐       │
│ │ ③ 左下  啟用□│ │ ④ 右下  啟用□│       │  ← 卡片列 2
│ │  ...         │ │  ...         │       │
│ └──────────────┘ └──────────────┘       │
├─────────────────────────────────────────┤
│ [啟動 N 個終端機]  [儲存設定]   ✓ 狀態  │  ← 操作列
└─────────────────────────────────────────┘
```

## 視窗尺寸

`820 × 520 px`（不可調整大小）

卡片高度：`152px`，卡片寬度：`(視窗寬 - 間距) / 2`

## 字型載入（main.rs）

啟動時依序嘗試：
1. `C:\Windows\Fonts\NotoSansTC-VF.ttf`
2. `C:\Windows\Fonts\msjh.ttc`（微軟正黑體）
3. `C:\Windows\Fonts\msyh.ttc`（微軟雅黑）
4. `C:\Windows\Fonts\mingliu.ttc`

找到後以 `egui::FontData` 追加為所有字族的備選（英文優先用內建字型）。

## 視窗尺寸

`820 × 540 px`（min/max 鎖死同尺寸，避免使用 `with_resizable(false)`）

**不要用 `with_resizable(false)`**：egui 0.28.1 在 `resizable(false)` 時 hit_test 有 unwrap panic，
改用 `with_min_inner_size` + `with_max_inner_size` 設成相同尺寸達到同效果。

## 修改 UI 的注意事項

- egui 0.28 使用 `Frame::none()` 而非 `Frame::new()`
- ComboBox 用 `from_id_source()` 而非 `from_id_salt()`（0.29+ 才有）
- **不要用 `add_enabled_ui`**：disabled widget 從 widget list 移除後，hit_test 持有舊參考會 panic
  → 改用 `TextEdit::interactive(bool)` 控制輸入框，ComboBox disabled 時改顯示靜態 Label
- 每幀都會呼叫 `ctx.set_visuals()`，確保主題不被 egui 預設覆蓋
