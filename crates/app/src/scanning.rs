//! Tarama ekrani: canli sayaclar ve iptal.

use crate::app::IskandilApp;
use crate::theme;
use crate::widgets;
use egui::{Align, Align2, Layout, Rect, RichText, Sense, Stroke, pos2, vec2};
use iskandil_core::format as fmt;
use std::time::Duration;

impl IskandilApp {
    pub fn scanning_ui(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let p = self.p;
        self.chrome_ui(ui);

        let Some(session) = &self.session else { return };
        let snap = session.progress();
        let target = session.root_display.clone();
        let threads = session.threads;
        let canceled = snap.canceled;

        egui::Panel::bottom("scan-status")
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
                    if canceled {
                        "iptal ediliyor · toplanan sonuç korunacak"
                    } else {
                        "okuma sürüyor · pencereyi kapatmak taramayı durdurur"
                    },
                    theme::font_mono(8.5),
                    p.ink_faint,
                );
                ui.painter().text(
                    pos2(rect.right() - 20.0, rect.center().y),
                    Align2::RIGHT_CENTER,
                    format!("{threads} iş parçacığı"),
                    theme::font_mono(8.5),
                    p.ink_faint,
                );
            });

        egui::CentralPanel::no_frame().show(ui, |ui| {
            let t = ctx.input(|i| i.time) as f32;
            let margin = 54.0;
            ui.add_space(52.0);
            ui.horizontal_top(|ui| {
                ui.add_space(margin);
                ui.allocate_ui_with_layout(
                    vec2((ui.available_width() - 40.0).max(600.0), 460.0),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.set_width(ui.available_width());
                        // Canli etiket
                        let (lrect, _) = ui.allocate_exact_size(vec2(340.0, 16.0), Sense::hover());
                        let pulse = 0.45 + 0.55 * ((t * 3.2).sin() * 0.5 + 0.5);
                        ui.painter().rect_filled(
                            Rect::from_min_size(
                                pos2(lrect.left(), lrect.center().y - 3.0),
                                vec2(6.0, 6.0),
                            ),
                            0.0,
                            p.accent.gamma_multiply(pulse),
                        );
                        ui.painter().text(
                            pos2(lrect.left() + 14.0, lrect.center().y),
                            Align2::LEFT_CENTER,
                            if canceled { "DURDURULUYOR" } else { "TARANIYOR" },
                            theme::font_mono(9.5),
                            p.ink_muted,
                        );

                        ui.add_space(18.0);
                        ui.scope(|ui| {
                            ui.set_max_width(ui.available_width() * 0.86);
                            ui.add(
                                egui::Label::new(
                                    RichText::new(&target)
                                        .font(theme::font_display(30.0))
                                        .color(p.ink),
                                )
                                .wrap(),
                            );
                        });
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("HEDEF YOL")
                                .font(theme::font_mono(8.5))
                                .color(p.ink_faint),
                        );

                        ui.add_space(42.0);
                        let stats = [
                            ("DOSYA", fmt::count(snap.files)),
                            ("VERİ", fmt::bytes(snap.bytes)),
                            ("KLASÖR", fmt::count(snap.dirs)),
                        ];
                        let width = ui.available_width().min(760.0);
                        let (row, _) = ui.allocate_exact_size(vec2(width, 64.0), Sense::hover());
                        let cell_w = width / 3.0;
                        for (i, (label, value)) in stats.iter().enumerate() {
                            let left = row.left() + cell_w * i as f32;
                            if i > 0 {
                                ui.painter().vline(
                                    left,
                                    row.y_range(),
                                    Stroke::new(1.0, p.hairline),
                                );
                            }
                            let pad = if i > 0 { 18.0 } else { 0.0 };
                            ui.painter().text(
                                pos2(left + pad, row.top() + 2.0),
                                Align2::LEFT_TOP,
                                *label,
                                theme::font_mono(8.5),
                                p.ink_faint,
                            );
                            ui.painter().text(
                                pos2(left + pad, row.top() + 20.0),
                                Align2::LEFT_TOP,
                                value.clone(),
                                theme::font_mono(26.0),
                                p.ink,
                            );
                        }

                        ui.add_space(26.0);
                        let (tr, _) =
                            ui.allocate_exact_size(vec2(width, 2.0), Sense::hover());
                        ui.painter().rect_filled(tr, 0.0, p.hairline);
                        let seg_w = tr.width() * 0.22;
                        let phase = (t * 0.22) % 1.0;
                        let x = tr.left() + phase * (tr.width() - seg_w);
                        ui.painter().rect_filled(
                            Rect::from_min_size(pos2(x, tr.top()), vec2(seg_w, 2.0)),
                            0.0,
                            p.accent.gamma_multiply(0.85),
                        );
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "geçen süre {}",
                                    fmt::duration(snap.elapsed)
                                ))
                                .font(theme::font_mono(9.0))
                                .color(p.ink_muted),
                            );
                            ui.add_space(16.0);
                            ui.label(
                                RichText::new(format!(
                                    "ortalama {}",
                                    fmt::rate_count(snap.files, snap.elapsed)
                                ))
                                .font(theme::font_mono(9.0))
                                .color(p.ink_faint),
                            );
                            ui.add_space(16.0);
                            ui.label(
                                RichText::new(format!(
                                    "ortalama {}",
                                    fmt::rate_bytes(snap.bytes, snap.elapsed)
                                ))
                                .font(theme::font_mono(9.0))
                                .color(p.ink_faint),
                            );
                        });

                        ui.add_space(28.0);
                        ui.horizontal(|ui| {
                            if widgets::ghost_button(ui, "DURDUR", &p).clicked() {
                                self.cancel_scan();
                            }
                            if snap.errors > 0 {
                                ui.add_space(14.0);
                                ui.label(
                                    RichText::new(format!(
                                        "{} öğe okunamadı (izin)",
                                        fmt::count(snap.errors)
                                    ))
                                    .font(theme::font_mono(9.0))
                                    .color(p.warning),
                                );
                            }
                        });

                        ui.add_space(40.0);
                        ui.label(
                            RichText::new(
                                "Dizinler paralel okunur; erişim engelli klasörler atlanır. İptal edersen o ana kadar bulunan ağaç korunur.",
                            )
                            .font(theme::font_body(12.0))
                            .color(p.ink_faint),
                        );
                    },
                );
            });
            ctx.request_repaint_after(Duration::from_millis(16));
        });
    }
}
