#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod branding_core;
mod config;
mod launcher;

fn main() -> eframe::Result {
    let cfg = config::load();
    let icon_rgba = branding_core::render_icon_rgba(64);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Open Terminal")
            .with_icon(egui::IconData {
                rgba: icon_rgba,
                width: 64,
                height: 64,
            })
            .with_inner_size([820.0, 560.0])
            .with_min_inner_size([820.0, 480.0])
            .with_resizable(true),
        ..Default::default()
    };

    eframe::run_native(
        "Open Terminal",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Ok(Box::new(app::App::new(cfg)))
        }),
    )
}

fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    // 嘗試從 Windows 字型目錄載入繁體中文字型
    let font_candidates = [
        r"C:\Windows\Fonts\NotoSansTC-VF.ttf",
        r"C:\Windows\Fonts\msjh.ttc", // 微軟正黑體
        r"C:\Windows\Fonts\msyh.ttc", // 微軟雅黑
        r"C:\Windows\Fonts\mingliu.ttc",
    ];

    let mut loaded = false;
    for path in &font_candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts
                .font_data
                .insert("cjk".to_owned(), egui::FontData::from_owned(data).into());
            // 插入到所有字族的最後備選（英文仍優先用預設）
            for family in fonts.families.values_mut() {
                family.push("cjk".to_owned());
            }
            loaded = true;
            break;
        }
    }

    if !loaded {
        // fallback：找 WSL 下 Windows 掛載路徑
        let wsl_candidates = [
            "/mnt/c/Windows/Fonts/NotoSansTC-VF.ttf",
            "/mnt/c/Windows/Fonts/msjh.ttc",
        ];
        for path in &wsl_candidates {
            if let Ok(data) = std::fs::read(path) {
                fonts
                    .font_data
                    .insert("cjk".to_owned(), egui::FontData::from_owned(data).into());
                for family in fonts.families.values_mut() {
                    family.push("cjk".to_owned());
                }
                break;
            }
        }
    }

    ctx.set_fonts(fonts);
}
