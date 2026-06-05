use egui::{Context, RichText, Color32, Vec2, Rounding};
use crate::config::{AppConfig, save};
use crate::launcher;

pub struct App {
    pub config: AppConfig,
    status: String,
    launching: bool,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            status: String::new(),
            launching: false,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Open Terminal");
            ui.add_space(8.0);
            ui.label("設定每個終端機視窗的名稱與啟動指令（留空則直接開啟 WSL）");
            ui.add_space(12.0);

            // 2x2 grid
            let cell_size = Vec2::new(280.0, 100.0);
            let indices = [[0, 1], [2, 3]];

            for row in &indices {
                ui.horizontal(|ui| {
                    for &i in row {
                        let pane = &mut self.config.panes[i];
                        egui::Frame::default()
                            .fill(Color32::from_gray(40))
                            .rounding(Rounding::same(6.0))
                            .inner_margin(egui::Margin::same(10.0))
                            .show(ui, |ui| {
                                ui.set_min_size(cell_size);
                                ui.label(RichText::new(format!("Pane {}", i + 1)).strong());
                                ui.add_space(4.0);
                                ui.horizontal(|ui| {
                                    ui.label("名稱：");
                                    ui.text_edit_singleline(&mut pane.label);
                                });
                                ui.horizontal(|ui| {
                                    ui.label("指令：");
                                    ui.text_edit_singleline(&mut pane.command);
                                });
                            });
                        ui.add_space(8.0);
                    }
                });
                ui.add_space(8.0);
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                let btn = ui.add_enabled(
                    !self.launching,
                    egui::Button::new(
                        RichText::new(if self.launching { "啟動中…" } else { "🚀  啟動四個終端機" })
                            .size(16.0)
                    )
                    .min_size(Vec2::new(200.0, 40.0)),
                );

                if btn.clicked() {
                    save(&self.config);
                    match launcher::launch(&self.config.panes) {
                        Ok(_) => self.status = "✅ 已啟動，視窗排列完成".into(),
                        Err(e) => self.status = format!("❌ 錯誤：{e}"),
                    }
                }

                if ui.button("💾  儲存設定").clicked() {
                    save(&self.config);
                    self.status = "設定已儲存".into();
                }
            });

            if !self.status.is_empty() {
                ui.add_space(8.0);
                ui.label(&self.status);
            }
        });
    }
}
