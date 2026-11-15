//! Iskandil gorsel dili.
//!
//! Metafor: olcum aleti / deniz haritasi. Grafik kagidi zemin, murekkep
//! tipografi, tek vermilyon vurgu. Paneller ve doseme kutulari kosesiz;
//! yalnizca etkilesimli kontrollerde 2 px yaricap. Golge, mor gradyan,
//! yumusak kart yigini yok.

use egui::{
    Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Stroke, TextStyle,
    Visuals,
};
use iskandil_core::settings::ThemeMode;
use std::sync::Arc;

pub const DISPLAY: &str = "display";
pub const MONO: &str = "mono";

pub fn font_display(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(DISPLAY.into()))
}

pub fn font_mono(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(MONO.into()))
}

pub fn font_body(size: f32) -> FontId {
    FontId::new(size, FontFamily::Proportional)
}

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    /// Sayfa zemini (grafik kagidi).
    pub bg: Color32,
    /// Yan paneller, ust serit.
    pub panel: Color32,
    /// Kontrol zemini.
    pub surface: Color32,
    pub surface_hi: Color32,
    pub ink: Color32,
    pub ink_muted: Color32,
    pub ink_faint: Color32,
    /// Baslik ikinci satiri gibi buyuk ama geri planda kalan metin.
    pub hero_faint: Color32,
    pub hairline: Color32,
    pub hairline_strong: Color32,
    pub accent: Color32,
    pub accent_hover: Color32,
    pub on_accent: Color32,
    pub warning: Color32,
    pub danger: Color32,
    /// Secili satir/doseme zemini.
    pub selection: Color32,
    /// Klasor dosemesi govdesi.
    pub folder: Color32,
    /// Doseme etiketleri.
    pub tile_text: Color32,
    pub tile_text_dim: Color32,
    /// Tema modu (renk secimi icin).
    pub mode: ThemeMode,
}

pub fn palette(mode: ThemeMode) -> Palette {
    match mode {
        ThemeMode::Light => Palette {
            bg: Color32::from_rgb(0xF1, 0xF2, 0xED),
            panel: Color32::from_rgb(0xF6, 0xF7, 0xF2),
            surface: Color32::from_rgb(0xE8, 0xEA, 0xE2),
            surface_hi: Color32::from_rgb(0xDF, 0xE2, 0xD8),
            ink: Color32::from_rgb(0x16, 0x19, 0x15),
            ink_muted: Color32::from_rgb(0x5C, 0x63, 0x5A),
            ink_faint: Color32::from_rgb(0x66, 0x6E, 0x63),
            hero_faint: Color32::from_rgb(0x7E, 0x86, 0x7B),
            hairline: Color32::from_rgb(0xD3, 0xD7, 0xCB),
            hairline_strong: Color32::from_rgb(0xB4, 0xBA, 0xAC),
            accent: Color32::from_rgb(0xB2, 0x3A, 0x2B),
            accent_hover: Color32::from_rgb(0x8E, 0x2C, 0x20),
            on_accent: Color32::from_rgb(0xF7, 0xF4, 0xEC),
            warning: Color32::from_rgb(0x9A, 0x6C, 0x1C),
            danger: Color32::from_rgb(0x8C, 0x2B, 0x21),
            selection: Color32::from_rgb(0xEB, 0xD5, 0xCF),
            folder: Color32::from_rgb(0xE2, 0xE5, 0xDC),
            tile_text: Color32::from_rgb(0x1A, 0x1D, 0x18),
            tile_text_dim: Color32::from_rgb(0x3A, 0x40, 0x38),
            mode,
        },
        ThemeMode::Dark => Palette {
            bg: Color32::from_rgb(0x0F, 0x12, 0x10),
            panel: Color32::from_rgb(0x14, 0x18, 0x15),
            surface: Color32::from_rgb(0x1B, 0x20, 0x1C),
            surface_hi: Color32::from_rgb(0x23, 0x29, 0x23),
            ink: Color32::from_rgb(0xE9, 0xEB, 0xE3),
            ink_muted: Color32::from_rgb(0xA2, 0xA9, 0x9E),
            ink_faint: Color32::from_rgb(0x8C, 0x94, 0x8A),
            hero_faint: Color32::from_rgb(0x9A, 0xA1, 0x97),
            hairline: Color32::from_rgb(0x2A, 0x30, 0x2A),
            hairline_strong: Color32::from_rgb(0x3C, 0x44, 0x3B),
            accent: Color32::from_rgb(0xE0, 0x60, 0x48),
            accent_hover: Color32::from_rgb(0xF2, 0x7A, 0x5F),
            on_accent: Color32::from_rgb(0x17, 0x0F, 0x0C),
            warning: Color32::from_rgb(0xD2, 0xA1, 0x4A),
            danger: Color32::from_rgb(0xE2, 0x69, 0x5C),
            selection: Color32::from_rgb(0x2C, 0x22, 0x1E),
            folder: Color32::from_rgb(0x19, 0x1E, 0x1A),
            tile_text: Color32::from_rgb(0xF2, 0xF1, 0xE9),
            tile_text_dim: Color32::from_rgb(0xC9, 0xCB, 0xC1),
            mode,
        },
    }
}

/// Dosya turu renkleri. Acik temada pastel zemin + koyu murekkep;
/// koyu temada doygun zemin + acik murekkep.
pub fn category_color(cat: iskandil_core::Category, mode: ThemeMode) -> Color32 {
    use iskandil_core::Category as C;
    match (mode, cat) {
        (ThemeMode::Light, C::Video) => Color32::from_rgb(0xD8, 0xA1, 0x8A),
        (ThemeMode::Light, C::Image) => Color32::from_rgb(0xDC, 0xC1, 0x87),
        (ThemeMode::Light, C::Audio) => Color32::from_rgb(0x9C, 0xC3, 0xB4),
        (ThemeMode::Light, C::Document) => Color32::from_rgb(0xA7, 0xB9, 0xD2),
        (ThemeMode::Light, C::Code) => Color32::from_rgb(0xB6, 0xCB, 0x99),
        (ThemeMode::Light, C::Archive) => Color32::from_rgb(0xC4, 0xAF, 0xD0),
        (ThemeMode::Light, C::App) => Color32::from_rgb(0xD3, 0xA5, 0xA9),
        (ThemeMode::Light, C::Other) => Color32::from_rgb(0xC2, 0xC5, 0xBA),
        (ThemeMode::Dark, C::Video) => Color32::from_rgb(0x8F, 0x4E, 0x38),
        (ThemeMode::Dark, C::Image) => Color32::from_rgb(0x87, 0x67, 0x1E),
        (ThemeMode::Dark, C::Audio) => Color32::from_rgb(0x35, 0x67, 0x57),
        (ThemeMode::Dark, C::Document) => Color32::from_rgb(0x45, 0x5E, 0x7C),
        (ThemeMode::Dark, C::Code) => Color32::from_rgb(0x5A, 0x74, 0x3C),
        (ThemeMode::Dark, C::Archive) => Color32::from_rgb(0x6D, 0x54, 0x7B),
        (ThemeMode::Dark, C::App) => Color32::from_rgb(0x83, 0x4B, 0x51),
        (ThemeMode::Dark, C::Other) => Color32::from_rgb(0x61, 0x67, 0x5D),
    }
}

/// Fontlari sistemden yukler; bulunamayanlar icin egui varsayilanlari kalir.
pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    let root = std::env::var_os("WINDIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from(r"C:\Windows"))
        .join("Fonts");

    let mut loaded_display = false;
    let mut loaded_mono = false;
    let mut loaded_body = false;
    let mut loaded_symbol = false;

    for (key, file) in [
        ("bahnschrift", "bahnschrift.ttf"),
        ("cascadia", "CascadiaMono.ttf"),
        ("segoe", "SegUIVar.ttf"),
        ("segoeui", "segoeui.ttf"),
        ("seguisym", "seguisym.ttf"),
    ] {
        let path = root.join(file);
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        fonts
            .font_data
            .insert(key.to_owned(), Arc::new(FontData::from_owned(bytes)));
        match key {
            "bahnschrift" => loaded_display = true,
            "cascadia" => loaded_mono = true,
            "segoe" | "segoeui" => loaded_body = true,
            "seguisym" => loaded_symbol = true,
            _ => {}
        }
    }

    let proportional_fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    let mono_fallback = fonts
        .families
        .get(&FontFamily::Monospace)
        .cloned()
        .unwrap_or_default();

    let mut body_chain: Vec<String> = Vec::new();
    if loaded_body {
        if fonts.font_data.contains_key("segoe") {
            body_chain.push("segoe".into());
        } else {
            body_chain.push("segoeui".into());
        }
    }
    body_chain.extend(proportional_fallback.iter().cloned());
    if loaded_symbol {
        body_chain.push("seguisym".into());
    }
    fonts
        .families
        .insert(FontFamily::Proportional, body_chain.clone());

    let mut mono_chain: Vec<String> = Vec::new();
    if loaded_mono {
        mono_chain.push("cascadia".into());
    }
    mono_chain.extend(mono_fallback.iter().cloned());
    fonts
        .families
        .insert(FontFamily::Monospace, mono_chain.clone());

    let display_chain = if loaded_display {
        vec!["bahnschrift".to_string(), body_chain[0].clone()]
    } else {
        body_chain.clone()
    };
    fonts
        .families
        .insert(FontFamily::Name(DISPLAY.into()), display_chain);
    fonts
        .families
        .insert(FontFamily::Name(MONO.into()), mono_chain);

    ctx.set_fonts(fonts);
}

/// Aktif temayi uygular.
pub fn apply(ctx: &egui::Context, mode: ThemeMode) {
    let p = palette(mode);
    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 5.0);
        style.spacing.interact_size.y = 28.0;
        style.spacing.indent = 14.0;
        style.spacing.scroll.bar_width = 6.0;
        style.spacing.scroll.bar_inner_margin = 2.0;
        style.spacing.scroll.bar_outer_margin = 0.0;
        style.spacing.scroll.floating = true;
        style.animation_time = 0.12;
        style.text_styles = [
            (TextStyle::Heading, font_display(24.0)),
            (TextStyle::Body, font_body(13.0)),
            (TextStyle::Button, font_body(13.0)),
            (TextStyle::Small, font_body(11.0)),
            (TextStyle::Monospace, font_mono(11.0)),
        ]
        .into();
    });
    ctx.set_visuals_of(egui::Theme::Light, visuals(&p, false));
    ctx.set_visuals_of(egui::Theme::Dark, visuals(&p, true));
    ctx.set_theme(match mode {
        ThemeMode::Light => egui::ThemePreference::Light,
        ThemeMode::Dark => egui::ThemePreference::Dark,
    });
}

fn visuals(p: &Palette, dark: bool) -> Visuals {
    let mut v = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    v.override_text_color = Some(p.ink);
    v.panel_fill = p.bg;
    v.window_fill = p.panel;
    v.extreme_bg_color = p.surface;
    v.faint_bg_color = p.surface;
    v.window_stroke = Stroke::new(1.0, p.hairline);
    v.window_corner_radius = CornerRadius::same(2);
    v.menu_corner_radius = CornerRadius::same(2);
    v.selection.bg_fill = p.selection;
    v.selection.stroke = Stroke::new(1.0, p.accent);
    v.hyperlink_color = p.accent;
    v.widgets.noninteractive.bg_fill = p.panel;
    v.widgets.noninteractive.weak_bg_fill = p.panel;
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0, p.hairline);
    v.widgets.noninteractive.fg_stroke = Stroke::new(1.0, p.ink_muted);
    v.widgets.inactive.bg_fill = p.surface;
    v.widgets.inactive.weak_bg_fill = p.surface;
    v.widgets.inactive.bg_stroke = Stroke::new(1.0, p.hairline);
    v.widgets.inactive.fg_stroke = Stroke::new(1.0, p.ink);
    v.widgets.hovered.bg_fill = p.surface_hi;
    v.widgets.hovered.weak_bg_fill = p.surface_hi;
    v.widgets.hovered.bg_stroke = Stroke::new(1.0, p.hairline_strong);
    v.widgets.hovered.fg_stroke = Stroke::new(1.0, p.ink);
    v.widgets.active.bg_fill = p.accent;
    v.widgets.active.weak_bg_fill = p.accent;
    v.widgets.active.bg_stroke = Stroke::new(1.0, p.accent_hover);
    v.widgets.active.fg_stroke = Stroke::new(1.0, p.on_accent);
    v.widgets.open.bg_fill = p.surface_hi;
    v.widgets.open.weak_bg_fill = p.surface_hi;
    v.widgets.open.bg_stroke = Stroke::new(1.0, p.hairline_strong);
    v.widgets.open.fg_stroke = Stroke::new(1.0, p.ink);
    v
}
