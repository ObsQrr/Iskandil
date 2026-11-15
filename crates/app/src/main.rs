#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod scanning;
mod start;
mod theme;
mod treemap_view;
mod widgets;
mod workspace;

use iskandil_core::settings::ThemeMode;
use std::path::PathBuf;
use std::sync::Arc;

fn main() -> eframe::Result<()> {
    let cli = parse_args();
    if let Some(path) = &cli.bench {
        run_bench(path);
        return Ok(());
    }
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default()
            .with_title(app::APP_TITLE)
            .with_app_id("iskandil")
            .with_inner_size(cli.size)
            .with_min_inner_size([1120.0, 700.0])
            .with_icon(Arc::new(app_icon())),
        ..Default::default()
    };
    let scan = cli.scan.clone();
    let shot = cli.shot.clone();
    let shot_now = cli.shot_now;
    let theme_override = cli.theme;
    let view = cli.view.clone();
    let search = cli.search.clone();
    let ask_delete_largest = cli.ask_delete_largest;
    eframe::run_native(
        "iskandil",
        options,
        Box::new(move |cc| {
            let app = app::IskandilApp::new(
                cc,
                app::LaunchOptions {
                    scan_path: scan.clone(),
                    shot: shot.clone(),
                    shot_now,
                    theme: theme_override,
                    view: view.clone(),
                    search: search.clone(),
                    ask_delete_largest,
                },
            );
            Ok(Box::new(app))
        }),
    )
}

struct Cli {
    scan: Option<String>,
    shot: Option<PathBuf>,
    theme: Option<ThemeMode>,
    size: [f32; 2],
    shot_now: bool,
    bench: Option<String>,
    view: Option<String>,
    search: Option<String>,
    ask_delete_largest: bool,
}

fn parse_args() -> Cli {
    let mut cli = Cli {
        scan: None,
        shot: None,
        theme: None,
        size: [1460.0, 920.0],
        shot_now: false,
        bench: None,
        view: None,
        search: None,
        ask_delete_largest: false,
    };
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--scan" => {
                if let Some(path) = args.next() {
                    cli.scan = Some(path);
                }
            }
            "--shot" => {
                if let Some(path) = args.next() {
                    cli.shot = Some(PathBuf::from(path));
                }
            }
            "--shot-now" => {
                cli.shot_now = true;
            }
            "--bench" => {
                if let Some(path) = args.next() {
                    cli.bench = Some(path);
                }
            }
            "--view" => {
                if let Some(path) = args.next() {
                    cli.view = Some(path);
                }
            }
            "--search" => {
                if let Some(text) = args.next() {
                    cli.search = Some(text);
                }
            }
            "--ask-delete-largest" => {
                cli.ask_delete_largest = true;
            }
            "--theme" => {
                cli.theme = match args.next().as_deref() {
                    Some("dark") | Some("koyu") => Some(ThemeMode::Dark),
                    Some("light") | Some("acik") | Some("açık") => Some(ThemeMode::Light),
                    _ => None,
                };
            }
            "--size" => {
                if let Some(value) = args.next() {
                    if let Some((w, h)) = value.split_once(['x', 'X', ',']) {
                        if let (Ok(w), Ok(h)) = (w.trim().parse::<f32>(), h.trim().parse::<f32>()) {
                            if w >= 900.0 && h >= 600.0 {
                                cli.size = [w, h];
                            }
                        }
                    }
                }
            }
            "--help" | "-h" => {
                println!(
                    "İskandil — disk derinlik ölçer\n\n\
                     Kullanım: iskandil [YOL] [--scan YOL] [--theme light|dark] [--size 1460x920]\n\
                     \n\
                     --scan YOL     Taramayı hemen başlat\n\
                     --shot PNG     Ekran görüntüsünü kaydet ve çık (geliştirme)\n\
                     --shot-now     Tarama bitmeden ekran görüntüsü al\n\
                     --bench YOL    Arayüz olmadan tara, ölçümleri yazdır\n\
                     --view YOL     Tarama sonrası bu alt klasöre gir\n\
                     --search METİN Tarama sonrası aramayı doldur\n\
                     --ask-delete-largest  En büyük dosyanın silme onayını aç\n\
                     --theme MODE   Açık/koyu temayı geçersiz kıl\n\
                     --size WxH     Pencere boyutu"
                );
                std::process::exit(0);
            }
            other if !other.starts_with("--") => {
                cli.scan = Some(other.to_string());
            }
            _ => {}
        }
    }
    cli
}

fn run_bench(path: &str) {
    use iskandil_core::scan::{ScanOptions, ScanSession};
    println!("hedef: {path}");
    let session = match ScanSession::start(path, None, ScanOptions::default()) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("hata: {e}");
            std::process::exit(1);
        }
    };
    let threads = session.threads;
    let tree = match session.wait() {
        Some(t) => t,
        None => {
            eprintln!("tarama sonucu alınamadı");
            std::process::exit(1);
        }
    };
    let secs = tree.elapsed.as_secs_f64();
    println!(
        "threads={threads} files={} dirs={} bytes={} errors={} elapsed={secs:.3}s files_per_sec={:.0} mb_per_sec={:.1}",
        tree.total_files(),
        tree.total_dirs(),
        tree.total_bytes(),
        tree.errors,
        tree.total_files() as f64 / secs.max(0.001),
        tree.total_bytes() as f64 / 1024.0 / 1024.0 / secs.max(0.001)
    );
}

/// Prosedurel uygulama ikonu: murekkep zemin, derinlik halkalari, iskandil ipi.
fn app_icon() -> egui::IconData {
    const SIZE: usize = 64;
    const SS: usize = 4;
    let mut rgba = vec![0u8; SIZE * SIZE * 4];
    let ink = [0x15u8, 0x18, 0x14];
    let paper = [0xF1u8, 0xF2, 0xED];
    let accent = [0xB2u8, 0x3A, 0x2B];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let mut sum = [0u32; 3];
            for sy in 0..SS {
                for sx in 0..SS {
                    let fx = (x as f32 + (sx as f32 + 0.5) / SS as f32) / SIZE as f32;
                    let fy = (y as f32 + (sy as f32 + 0.5) / SS as f32) / SIZE as f32;
                    let mut color = ink;
                    // Esmerkezli derinlik halkalari
                    let dx = fx - 0.16;
                    let dy = fy - 0.88;
                    let d = (dx * dx + dy * dy).sqrt();
                    for radius in [0.30f32, 0.50, 0.70] {
                        if (d - radius).abs() < 0.018 {
                            color = paper;
                        }
                    }
                    // Iskandil ipi ve agirligi
                    if (fx - 0.72).abs() < 0.022 && (0.10..0.92).contains(&fy) {
                        color = accent;
                    }
                    if (fx - 0.72).abs() < 0.055 && (fy - 0.82).abs() < 0.055 {
                        color = accent;
                    }
                    // Kenar cercevesi
                    if fx < 0.012 || fx > 0.988 || fy < 0.012 || fy > 0.988 {
                        color = [0x0C, 0x0E, 0x0B];
                    }
                    for c in 0..3 {
                        sum[c] += color[c] as u32;
                    }
                }
            }
            let idx = (y * SIZE + x) * 4;
            let n = (SS * SS) as u32;
            rgba[idx] = (sum[0] / n) as u8;
            rgba[idx + 1] = (sum[1] / n) as u8;
            rgba[idx + 2] = (sum[2] / n) as u8;
            rgba[idx + 3] = 255;
        }
    }
    egui::IconData {
        rgba,
        width: SIZE as u32,
        height: SIZE as u32,
    }
}
