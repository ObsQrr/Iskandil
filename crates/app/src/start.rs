//! Baslangic ekrani: suruculer ve hedef yol.

use crate::app::IskandilApp;
use crate::theme;
use crate::widgets;
use egui::{Align, Align2, Id, Layout, Rect, RichText, ScrollArea, Sense, Stroke, Ui, pos2, vec2};
use iskandil_core::format as fmt;
use iskandil_core::scan::ScanOptions;

impl IskandilApp {
    pub fn start_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let p = self.p;
        let threads = ScanOptions::default().threads;

        egui::Panel::bottom("start-status")
            .exact_size(32.0)
            .resizable(false)
            .frame(egui::Frame::NONE.fill(p.panel))
            .show(ui, |ui| {
                let rect = ui.max_rect();
                ui.painter().hline(
                    rect.x_range(),
                    rect.top() + 0.5,
                    Stroke::new(1.0, p.hairline),
                );
                ui.painter().text(
                    pos2(rect.left() + 20.0, rect.center().y),
                    Align2::LEFT_CENTER,
                    format!("{threads} iş parçacığı · paralel dizin taraması · NTFS"),
                    theme::font_mono(8.5),
                    p.ink_faint,
                );
                ui.painter().text(
                    pos2(rect.right() - 20.0, rect.center().y),
                    Align2::RIGHT_CENTER,
                    "v0.1.0",
                    theme::font_mono(8.5),
                    p.ink_faint,
                );
                let theme_label = if p.mode == iskandil_core::settings::ThemeMode::Dark {
                    "AÇIK"
                } else {
                    "KOYU"
                };
                let toggle_rect = Rect::from_min_size(
                    pos2(rect.right() - 96.0, rect.center().y - 10.0),
                    vec2(50.0, 20.0),
                );
                let toggle = ui.interact(
                    toggle_rect,
                    ui.scope_id().with("start-theme"),
                    Sense::click(),
                );
                let color = if toggle.hovered() {
                    p.accent
                } else {
                    p.ink_muted
                };
                ui.painter().text(
                    pos2(toggle_rect.center().x, rect.center().y),
                    Align2::CENTER_CENTER,
                    theme_label,
                    theme::font_mono(8.5),
                    color,
                );
                if toggle.hovered() {
                    ui.painter().hline(
                        toggle_rect.x_range(),
                        toggle_rect.bottom() - 2.0,
                        Stroke::new(1.0, p.accent),
                    );
                }
                if toggle.clicked() {
                    self.toggle_theme(ctx);
                }
            });

        egui::CentralPanel::no_frame().show(ui, |ui| {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let margin = 54.0;
                    ui.add_space(30.0);
                    // Ust marka seridi
                    {
                        let (rect, _) = ui.allocate_exact_size(
                            vec2(ui.available_width() - margin * 2.0, 22.0),
                            Sense::hover(),
                        );
                        ui.add_space(0.0);
                        let r = Rect::from_min_size(
                            pos2(rect.left() + margin, rect.top()),
                            vec2(rect.width(), rect.height()),
                        );
                        ui.painter().text(
                            pos2(r.left(), r.center().y),
                            Align2::LEFT_CENTER,
                            "İSKANDİL",
                            theme::font_display(12.0),
                            p.ink,
                        );
                        ui.painter().hline(
                            r.left() + 78.0..=r.right(),
                            r.center().y,
                            Stroke::new(1.0, p.hairline),
                        );
                        ui.painter().text(
                            pos2(r.right(), r.center().y),
                            Align2::RIGHT_CENTER,
                            "disk derinlik ölçer",
                            theme::font_mono(8.5),
                            p.ink_faint,
                        );
                    }

                    ui.add_space(36.0);
                    let avail_w = (ui.available_width() - margin * 2.0).max(720.0);
                    let left_w = (avail_w * 0.47).max(430.0);
                    let gap = 56.0;
                    let right_w = (avail_w - left_w - gap).max(300.0);

                    let mut scan_target: Option<String> = None;
                    let mut browse = false;
                    let mut go = false;

                    ui.horizontal_top(|ui| {
                        ui.add_space(margin);
                        ui.allocate_ui_with_layout(
                            vec2(left_w, 470.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                ui.set_width(left_w);
                                widgets::hero_text(ui, &p, "Diski ölç.", 50.0, p.ink);
                                widgets::hero_text(ui, &p, "Derinliği gör.", 50.0, p.hero_faint);
                                ui.add_space(16.0);
                                ui.scope(|ui| {
                                    ui.set_max_width(left_w * 0.86);
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(
                                                "Bir sürücü seç ya da klasör yolu ver. Hangi dosyanın ne kadar yer tuttuğunu gör.",
                                            )
                                            .font(theme::font_body(13.5))
                                            .color(p.ink_muted),
                                        )
                                        .wrap(),
                                    );
                                });
                                ui.add_space(26.0);
                                ui.label(
                                    RichText::new("HEDEF")
                                        .font(theme::font_mono(9.0))
                                        .color(p.ink_muted),
                                );
                                ui.add_space(6.0);

                                ui.horizontal(|ui| {
                                    let input_w = left_w - 190.0;
                                    let focused = ctx.memory(|m| m.has_focus(Id::unique("iskandil-path")));
                                    egui::Frame::NONE
                                        .fill(p.bg)
                                        .stroke(Stroke::new(
                                            if focused { 1.5 } else { 1.0 },
                                            if focused { p.accent } else { p.hairline_strong },
                                        ))
                                        .corner_radius(egui::CornerRadius::same(2))
                                        .inner_margin(egui::Margin::symmetric(10, 7))
                                        .show(ui, |ui| {
                                            ui.set_width(input_w - 20.0);
                                            let resp = ui.add(
                                                egui::TextEdit::singleline(&mut self.path_input)
                                                    .id(Id::unique("iskandil-path"))
                                                    .font(theme::font_mono(11.5))
                                                    .text_color(p.ink)
                                                    .hint_text(
                                                        RichText::new("C:\\Users\\...")
                                                            .font(theme::font_mono(11.0))
                                                            .color(p.ink_faint),
                                                    )
                                                    .frame(egui::Frame::NONE)
                                                    .margin(egui::Margin::ZERO)
                                                    .desired_width(f32::INFINITY),
                                            );
                                            let enter =
                                                ui.input(|i| i.key_pressed(egui::Key::Enter));
                                            if enter && (resp.has_focus() || resp.lost_focus()) {
                                                go = true;
                                            }
                                        });
                                    ui.add_space(6.0);
                                    if widgets::ghost_button(ui, "GÖZAT", &p).clicked() {
                                        browse = true;
                                    }
                                    ui.add_space(4.0);
                                    if widgets::primary_button(ui, "TARA →", &p).clicked() {
                                        go = true;
                                    }
                                });

                                ui.add_space(10.0);
                                if let Some(err) = &self.scan_error {
                                    ui.horizontal(|ui| {
                                        let (r, _) = ui.allocate_exact_size(
                                            vec2(14.0, 14.0),
                                            Sense::hover(),
                                        );
                                        widgets::warning_glyph(ui.painter(), r, p.danger);
                                        ui.add_space(4.0);
                                        ui.label(
                                            RichText::new(err)
                                                .font(theme::font_body(11.5))
                                                .color(p.danger),
                                        );
                                    });
                                } else {
                                    ui.label(
                                        RichText::new(
                                            "Enter ile başlat · tarama iptal edilirse o ana kadarki sonuç korunur",
                                        )
                                        .font(theme::font_mono(8.5))
                                        .color(p.ink_faint),
                                    );
                                }
                            },
                        );

                        ui.add_space(gap);
                        ui.allocate_ui_with_layout(
                            vec2(right_w, 470.0),
                            Layout::top_down(Align::Min),
                            |ui| {
                                ui.set_width(right_w);
                                ui.horizontal(|ui| {
                                    widgets::section_label(ui, &p, "SÜRÜCÜLER");
                                });
                                ui.add_space(4.0);
                                if self.drives.is_empty() {
                                    ui.add_space(10.0);
                                    ui.label(
                                        RichText::new("Erişilebilir sürücü bulunamadı.")
                                            .font(theme::font_body(12.0))
                                            .color(p.ink_muted),
                                    );
                                }
                                for (i, drive) in self.drives.iter().enumerate() {
                                    let letter = drive.letter.clone();
                                    let root = drive.root.clone();
                                    let label = if drive.label.is_empty() {
                                        drive.kind.label_tr().to_string()
                                    } else {
                                        drive.label.clone()
                                    };
                                    let line2 = format!(
                                        "{} dolu · {} boş",
                                        fmt::bytes(drive.used()),
                                        fmt::bytes(drive.free)
                                    );
                                    let pct = fmt::percent(drive.used(), drive.total);
                                    let fs = if drive.fs.is_empty() {
                                        drive.kind.label_tr().to_string()
                                    } else {
                                        drive.fs.clone()
                                    };
                                    let resp = drive_row(ui, &p, &letter, &label, &line2, &pct, &fs, drive.used_fraction(), i);
                                    if resp.clicked() {
                                        scan_target = Some(root);
                                    }
                                }
                                ui.add_space(12.0);
                                ui.label(
                                    RichText::new("veya soldaki alandan bir klasör yolu gir")
                                        .font(theme::font_mono(8.5))
                                        .color(p.ink_faint),
                                );
                            },
                        );
                    });

                    ui.add_space(30.0);

                    if go {
                        let path = self.path_input.clone();
                        self.start_scan(ctx, &path);
                    }
                    if browse {
                        if let Some(dir) = rfd::FileDialog::new()
                            .set_title("Taranacak klasörü seç")
                            .pick_folder()
                        {
                            self.path_input = dir.display().to_string();
                            self.scan_error = None;
                        }
                    }
                    if let Some(root) = scan_target {
                        self.start_scan(ctx, &root);
                    }
                });
        });
    }
}

#[allow(clippy::too_many_arguments)]
fn drive_row(
    ui: &mut Ui,
    p: &crate::theme::Palette,
    letter: &str,
    label: &str,
    line2: &str,
    pct: &str,
    fs: &str,
    used_fraction: f32,
    index: usize,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 62.0), Sense::click());
    let anim =
        ui.ctx()
            .animate_bool_with_time(Id::unique(("drive-row", index)), resp.hovered(), 0.12);
    let painter = ui.painter();
    if anim > 0.01 {
        painter.rect_filled(rect, 0.0, p.bg.lerp_to_gamma(p.panel, anim));
    }
    painter.hline(
        rect.x_range(),
        rect.bottom() - 0.5,
        Stroke::new(1.0, p.hairline),
    );

    painter.text(
        pos2(rect.left() + 2.0, rect.top() + 16.0),
        Align2::LEFT_TOP,
        letter,
        theme::font_display(22.0),
        p.ink,
    );
    painter.text(
        pos2(rect.left() + 62.0, rect.top() + 12.0),
        Align2::LEFT_TOP,
        label,
        theme::font_body(13.0),
        p.ink,
    );
    let label_w = painter
        .layout_no_wrap(label.to_owned(), theme::font_body(13.0), p.ink)
        .size()
        .x;
    painter.text(
        pos2(rect.left() + 68.0 + label_w, rect.top() + 14.0),
        Align2::LEFT_TOP,
        fs,
        theme::font_mono(8.5),
        p.ink_faint,
    );
    painter.text(
        pos2(rect.left() + 62.0, rect.top() + 32.0),
        Align2::LEFT_TOP,
        line2,
        theme::font_mono(9.5),
        p.ink_muted,
    );
    painter.text(
        pos2(rect.right() - 2.0, rect.top() + 13.0),
        Align2::RIGHT_TOP,
        pct,
        theme::font_mono(10.0),
        p.ink_muted,
    );

    // Kapasite cubugu
    let bar = Rect::from_min_size(
        pos2(rect.left() + 62.0, rect.bottom() - 14.0),
        vec2(rect.width() - 64.0, 2.0),
    );
    let fill = p.ink_muted.lerp_to_gamma(p.accent, anim);
    widgets::progress_bar(painter, bar, used_fraction, fill, p.hairline);

    // Ok
    let arrow_color = p.ink_faint.lerp_to_gamma(p.accent, anim);
    let c = pos2(rect.right() - 12.0, rect.center().y + 4.0);
    painter.line_segment(
        [pos2(c.x - 5.0, c.y), pos2(c.x + 3.0, c.y)],
        Stroke::new(1.2, arrow_color),
    );
    painter.line_segment(
        [pos2(c.x - 1.0, c.y - 4.0), pos2(c.x + 3.0, c.y)],
        Stroke::new(1.2, arrow_color),
    );
    painter.line_segment(
        [pos2(c.x - 1.0, c.y + 4.0), pos2(c.x + 3.0, c.y)],
        Stroke::new(1.2, arrow_color),
    );

    resp.on_hover_cursor(egui::CursorIcon::PointingHand)
}
