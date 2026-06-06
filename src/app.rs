use crate::config::{export, import, list_terminal_profiles, save, AppConfig, TerminalProfile};
use crate::launcher;
use crate::branding_core;
use egui::{
    Align, Color32, Context, FontId, Frame, Layout, Margin, Pos2, RichText, Rounding, Stroke,
    TextureHandle, TextureOptions, Vec2,
};

// ── 調色盤 ──────────────────────────────────────────────────────────────────
const BG: Color32 = Color32::from_rgb(15, 23, 42);
const CARD: Color32 = Color32::from_rgb(30, 41, 59);
const CARD_DISABLED: Color32 = Color32::from_rgb(20, 28, 40);
const BORDER: Color32 = Color32::from_rgb(71, 85, 105);
const BORDER_FOCUS: Color32 = Color32::from_rgb(34, 197, 94);
const TEXT: Color32 = Color32::from_rgb(248, 250, 252);
const TEXT_MUTED: Color32 = Color32::from_rgb(100, 116, 139);
const TEXT_DIM: Color32 = Color32::from_rgb(55, 65, 81);
const ACCENT: Color32 = Color32::from_rgb(34, 197, 94);
const ACCENT_DIM: Color32 = Color32::from_rgb(21, 128, 61);
const BTN_SEC: Color32 = Color32::from_rgb(51, 65, 85);
const STATUS_ERR: Color32 = Color32::from_rgb(239, 68, 68);
const STATUS_OK: Color32 = Color32::from_rgb(34, 197, 94);

const PAD_H: f32 = 14.0; // inner_margin vertical
const PAD_W: f32 = 20.0; // inner_margin horizontal
                         // ────────────────────────────────────────────────────────────────────────────

pub struct App {
    pub config: AppConfig,
    profiles: Vec<TerminalProfile>,
    status: String,
    status_ok: bool,
    launching: bool,
    logo: Option<TextureHandle>,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let profiles = list_terminal_profiles();
        Self {
            config,
            profiles,
            status: String::new(),
            status_ok: true,
            launching: false,
            logo: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = BG;
        visuals.window_fill = BG;
        visuals.override_text_color = Some(TEXT);
        visuals.widgets.noninteractive.bg_fill = CARD;
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
        visuals.widgets.inactive.bg_fill = BTN_SEC;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, BORDER);
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 55, 78);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.5, BORDER_FOCUS);
        visuals.widgets.active.bg_fill = ACCENT_DIM;
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, ACCENT);
        visuals.selection.bg_fill = Color32::from_rgb(34, 197, 94).gamma_multiply(0.25);
        visuals.selection.stroke = Stroke::new(1.0, ACCENT);
        ctx.set_visuals(visuals);

        egui::TopBottomPanel::top("header")
            .frame(
                Frame::none()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(PAD_W, PAD_H)),
            )
            .show(ctx, |ui| {
                self.draw_header(ui);
            });

        egui::TopBottomPanel::bottom("footer")
            .frame(
                Frame::none()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(PAD_W, PAD_H)),
            )
            .show(ctx, |ui| {
                self.draw_footer(ui);
            });

        egui::CentralPanel::default()
            .frame(
                Frame::none()
                    .fill(BG)
                    .inner_margin(Margin::symmetric(PAD_W, PAD_H)),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.draw_cards(ui);
                    });
            });
    }
}

impl App {
    fn draw_header(&mut self, ui: &mut egui::Ui) {
        // 第一列：標題 + 雙螢幕 + 刷新
        ui.horizontal(|ui| {
            let texture = self.logo.get_or_insert_with(|| {
                let rgba = branding_core::render_icon_rgba(64);
                let image = egui::ColorImage::from_rgba_unmultiplied([64, 64], &rgba);
                ui.ctx()
                    .load_texture("app-logo", image, TextureOptions::LINEAR)
            });
            ui.add(
                egui::Image::new((texture.id(), Vec2::new(20.0, 20.0)))
                    .sense(egui::Sense::hover()),
            );
            ui.add_space(8.0);
            ui.label(
                RichText::new("Open Terminal")
                    .font(FontId::proportional(18.0))
                    .color(TEXT)
                    .strong(),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("↻")
                                .font(FontId::proportional(13.0))
                                .color(TEXT_MUTED),
                        )
                        .fill(Color32::TRANSPARENT)
                        .stroke(Stroke::NONE)
                        .min_size(Vec2::new(24.0, 20.0)),
                    )
                    .on_hover_text("重新整理 WSL distro 清單")
                    .clicked()
                {
                    self.profiles = list_terminal_profiles();
                }
                ui.add_space(8.0);
                ui.checkbox(&mut self.config.dual_monitor, "");
                ui.label(
                    RichText::new("雙螢幕")
                        .font(FontId::proportional(12.0))
                        .color(if self.config.dual_monitor { ACCENT } else { TEXT_MUTED }),
                );
            });
        });

        ui.add_space(6.0);

        // 第二列：預設 Profile + 匯入 / 匯出
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("預設 Profile")
                    .font(FontId::proportional(11.5))
                    .color(TEXT_MUTED),
            );
            ui.add_space(4.0);

            let default_display = {
                let key = self.config.default_profile.trim();
                if key.is_empty() {
                    "預設".to_string()
                } else {
                    self.profiles
                        .iter()
                        .find(|p| p.guid == key || p.name == key)
                        .map(|p| self.profile_display_name(p))
                        .unwrap_or_else(|| key.to_string())
                }
            };
            let profile_options: Vec<(String, String)> = self
                .profiles
                .iter()
                .map(|p| (p.guid.clone(), self.profile_display_name(p)))
                .collect();

            egui::ComboBox::from_id_source("default_profile")
                .selected_text(
                    RichText::new(&default_display).font(FontId::proportional(12.0)),
                )
                .width(180.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.config.default_profile,
                        String::new(),
                        RichText::new("預設").font(FontId::proportional(12.0)),
                    );
                    for (guid, label) in &profile_options {
                        ui.selectable_value(
                            &mut self.config.default_profile,
                            guid.clone(),
                            RichText::new(label).font(FontId::proportional(12.0)),
                        );
                    }
                });

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                // 匯出
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("匯出").font(FontId::proportional(12.0)).color(TEXT),
                        )
                        .fill(BTN_SEC)
                        .stroke(Stroke::new(1.0, BORDER))
                        .rounding(Rounding::same(6.0))
                        .min_size(Vec2::new(52.0, 26.0)),
                    )
                    .clicked()
                {
                    self.do_export();
                }
                ui.add_space(4.0);
                // 匯入
                if ui
                    .add(
                        egui::Button::new(
                            RichText::new("匯入").font(FontId::proportional(12.0)).color(TEXT),
                        )
                        .fill(BTN_SEC)
                        .stroke(Stroke::new(1.0, BORDER))
                        .rounding(Rounding::same(6.0))
                        .min_size(Vec2::new(52.0, 26.0)),
                    )
                    .clicked()
                {
                    self.do_import();
                }
            });
        });

        self.draw_separator(ui);
    }

    fn do_import(&mut self) {
        // rfd 是跨平台原生檔案對話框
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON 設定檔", &["json"])
            .pick_file()
        {
            match import(&path) {
                Some(cfg) => {
                    self.config = cfg;
                    save(&self.config);
                    self.status = format!("已匯入：{}", path.display());
                    self.status_ok = true;
                }
                None => {
                    self.status = "匯入失敗：格式不正確".into();
                    self.status_ok = false;
                }
            }
        }
    }

    fn do_export(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("JSON 設定檔", &["json"])
            .set_file_name("open-terminal-config.json")
            .save_file()
        {
            match export(&self.config, &path) {
                Ok(_) => {
                    self.status = format!("已匯出：{}", path.display());
                    self.status_ok = true;
                }
                Err(e) => {
                    self.status = format!("匯出失敗：{e}");
                    self.status_ok = false;
                }
            }
        }
    }

    fn draw_cards(&mut self, ui: &mut egui::Ui) {
        let available_w = ui.available_width().max(320.0);
        let columns = if available_w >= 760.0 { 2 } else { 1 };
        let gap = 8.0;
        let card_w = if columns == 2 {
            (available_w - gap) / 2.0
        } else {
            available_w
        };

        self.draw_card_group(ui, false, columns, card_w, gap);

        if self.config.dual_monitor {
            ui.add_space(4.0);
            self.draw_separator(ui);

            ui.label(
                RichText::new("螢幕二")
                    .font(FontId::proportional(11.5))
                    .color(TEXT_MUTED),
            );
            ui.add_space(4.0);

            self.draw_card_group(ui, true, columns, card_w, gap);
        }
    }

    fn draw_card_group(
        &mut self,
        ui: &mut egui::Ui,
        is_second: bool,
        columns: usize,
        card_w: f32,
        gap: f32,
    ) {
        let pane_labels = ["① 左上", "② 右上", "③ 左下", "④ 右下"];
        let rows = pane_labels.len().div_ceil(columns);

        for row in 0..rows {
            ui.horizontal(|ui| {
                for col in 0..columns {
                    let i = row * columns + col;
                    if i >= pane_labels.len() {
                        break;
                    }
                    ui.allocate_ui_with_layout(
                        Vec2::new(card_w, 0.0),
                        Layout::top_down(Align::Min),
                        |ui| {
                            ui.set_min_width(card_w);
                            ui.set_max_width(card_w);
                            self.draw_card(ui, i, pane_labels[i], is_second);
                        },
                    );
                    if col + 1 < columns {
                        ui.add_space(gap);
                    }
                }
            });
            ui.add_space(gap);
        }
    }

    fn draw_card(&mut self, ui: &mut egui::Ui, i: usize, pane_label: &str, is_second: bool) {
        let pane = if is_second { &self.config.panes2[i] } else { &self.config.panes[i] };
        let enabled = pane.enabled;
        let label_val = pane.label.clone();
        let command_val = pane.command.clone();
        let profile_key = pane.profile.clone();

        let profile_display = {
            let key = profile_key.trim();
            if key.is_empty() {
                "預設".to_string()
            } else {
                self.profiles
                    .iter()
                    .find(|p| p.guid == key || p.name == key)
                    .map(|p| self.profile_display_name(p))
                    .unwrap_or_else(|| key.to_string())
            }
        };
        let profile_options: Vec<(String, String)> = self
            .profiles
            .iter()
            .map(|p| (p.guid.clone(), self.profile_display_name(p)))
            .collect();

        // combobox id 需區分兩組，避免 egui id 衝突
        let combo_id = if is_second { format!("distro2_{i}") } else { format!("distro_{i}") };

        Frame::none()
            .fill(if enabled { CARD } else { CARD_DISABLED })
            .rounding(Rounding::same(10.0))
            .stroke(Stroke::new(
                1.0,
                if enabled { BORDER } else { Color32::from_rgb(38, 48, 64) },
            ))
            .inner_margin(Margin::same(12.0))
            .show(ui, |ui| {
                let field_label_w = 42.0;

                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(pane_label)
                            .font(FontId::proportional(11.0))
                            .color(if enabled { ACCENT } else { TEXT_DIM }),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new("啟用")
                                .font(FontId::proportional(10.5))
                                .color(TEXT_DIM),
                        );
                        let pane_mut = if is_second { &mut self.config.panes2[i] } else { &mut self.config.panes[i] };
                        ui.checkbox(&mut pane_mut.enabled, "");
                    });
                });

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.add_sized(
                        [field_label_w, 14.0],
                        egui::Label::new(
                            RichText::new("名稱")
                                .font(FontId::proportional(11.0))
                                .color(if enabled { TEXT_MUTED } else { TEXT_DIM }),
                        ),
                    );
                    let input_w = (ui.available_width() - 4.0).max(120.0);
                    let mut val = label_val;
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut val)
                                .hint_text("Terminal 名稱")
                                .font(FontId::proportional(12.0))
                                .desired_width(input_w)
                                .interactive(enabled)
                                .margin(Margin::symmetric(6.0, 4.0)),
                        )
                        .changed()
                    {
                        let pane_mut = if is_second { &mut self.config.panes2[i] } else { &mut self.config.panes[i] };
                        pane_mut.label = val;
                    }
                });

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.add_sized(
                        [field_label_w, 14.0],
                        egui::Label::new(
                            RichText::new("Profile")
                                .font(FontId::proportional(11.0))
                                .color(if enabled { TEXT_MUTED } else { TEXT_DIM }),
                        ),
                    );
                    let input_w = (ui.available_width() - 4.0).max(120.0);
                    if enabled {
                        let pane_mut = if is_second { &mut self.config.panes2[i] } else { &mut self.config.panes[i] };
                        egui::ComboBox::from_id_source(&combo_id)
                            .selected_text(RichText::new(&profile_display).font(FontId::proportional(12.0)))
                            .width(input_w)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut pane_mut.profile,
                                    String::new(),
                                    RichText::new("預設").font(FontId::proportional(12.0)),
                                );
                                for (guid, label) in &profile_options {
                                    ui.selectable_value(
                                        &mut pane_mut.profile,
                                        guid.clone(),
                                        RichText::new(label).font(FontId::proportional(12.0)),
                                    );
                                }
                            });
                    } else {
                        ui.label(
                            RichText::new(&profile_display)
                                .font(FontId::proportional(12.0))
                                .color(TEXT_DIM),
                        );
                    }
                });

                ui.add_space(5.0);

                ui.horizontal(|ui| {
                    ui.add_sized(
                        [field_label_w, 14.0],
                        egui::Label::new(
                            RichText::new("指令")
                                .font(FontId::proportional(11.0))
                                .color(if enabled { TEXT_MUTED } else { TEXT_DIM }),
                        ),
                    );
                    let input_w = (ui.available_width() - 4.0).max(120.0);
                    let mut val = command_val;
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut val)
                                .hint_text("留空直接開啟 WSL")
                                .font(FontId::monospace(11.5))
                                .desired_width(input_w)
                                .interactive(enabled)
                                .margin(Margin::symmetric(6.0, 4.0)),
                        )
                        .changed()
                    {
                        let pane_mut = if is_second { &mut self.config.panes2[i] } else { &mut self.config.panes[i] };
                        pane_mut.command = val;
                    }
                });
            });
    }

    fn draw_footer(&mut self, ui: &mut egui::Ui) {
        self.draw_separator(ui);
        ui.horizontal(|ui| {
            let enabled_count = self.config.panes.iter().filter(|p| p.enabled).count()
                + if self.config.dual_monitor {
                    self.config.panes2.iter().filter(|p| p.enabled).count()
                } else {
                    0
                };
            let can_launch = !self.launching && enabled_count > 0;
            let launch_label = if self.launching {
                "啟動中…".to_string()
            } else {
                format!("啟動 {} 個終端機", enabled_count)
            };

            if ui
                .add(
                    egui::Button::new(
                        RichText::new(&launch_label)
                            .font(FontId::proportional(13.5))
                            .color(if can_launch {
                                Color32::from_rgb(15, 23, 42)
                            } else {
                                TEXT_DIM
                            }),
                    )
                    .fill(if can_launch { ACCENT } else { BTN_SEC })
                    .stroke(Stroke::NONE)
                    .rounding(Rounding::same(7.0))
                    .min_size(Vec2::new(155.0, 34.0)),
                )
                .clicked()
                && can_launch
            {
                save(&self.config);
                match launcher::launch(&self.config) {
                    Ok(_) => {
                        self.status = "已啟動，視窗排列完成".into();
                        self.status_ok = true;
                    }
                    Err(e) => {
                        self.status = format!("錯誤：{e}");
                        self.status_ok = false;
                    }
                }
            }

            ui.add_space(8.0);

            if ui
                .add(
                    egui::Button::new(
                        RichText::new("儲存設定")
                            .font(FontId::proportional(13.0))
                            .color(TEXT),
                    )
                    .fill(BTN_SEC)
                    .stroke(Stroke::new(1.0, BORDER))
                    .rounding(Rounding::same(7.0))
                    .min_size(Vec2::new(88.0, 34.0)),
                )
                .clicked()
            {
                save(&self.config);
                self.status = "設定已儲存".into();
                self.status_ok = true;
            }

            if !self.status.is_empty() {
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let (color, prefix) = if self.status_ok {
                        (STATUS_OK, "✓  ")
                    } else {
                        (STATUS_ERR, "✗  ")
                    };
                    ui.label(
                        RichText::new(format!("{}{}", prefix, self.status))
                            .font(FontId::proportional(11.5))
                            .color(color),
                    );
                });
            }
        });
    }

    fn draw_separator(&self, ui: &mut egui::Ui) {
        ui.add_space(3.0);
        let sep_y = ui.cursor().top();
        ui.painter().line_segment(
            [
                Pos2::new(ui.min_rect().left(), sep_y),
                Pos2::new(ui.min_rect().right(), sep_y),
            ],
            Stroke::new(1.0, BORDER),
        );
        ui.add_space(10.0);
    }

    fn profile_display_name(&self, profile: &TerminalProfile) -> String {
        let duplicate_count = self
            .profiles
            .iter()
            .filter(|candidate| candidate.name == profile.name)
            .count();

        if duplicate_count > 1 {
            if let Some(source) = &profile.source {
                return format!("{} ({})", profile.name, source);
            }
        }

        profile.name.clone()
    }
}
