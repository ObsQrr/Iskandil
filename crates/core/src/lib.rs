//! Iskandil cekirdegi.
//!
//! * `scan` - paralel dizin tarayicisi (canli ilerleme, iptal, kismi sonuc)
//! * `model` - arenada tutulan dosya agaci, toplamlar, en buyuk dosyalar
//! * `treemap` - saf squarified treemap yerlesimi
//! * `categories` - uzantiya gore dosya turleri
//! * `format` - Turkce sayi/boyut/sure bicimlendirme
//! * `settings` - kalici ayarlar
//! * `sys` - surucu listesi ve geri donusum kutusu (Windows)

pub mod categories;
pub mod format;
pub mod model;
pub mod scan;
pub mod settings;
pub mod treemap;

#[cfg(windows)]
pub mod sys;

pub use categories::Category;
pub use model::{NONE, Node, Tree};
pub use scan::{ProgressSnapshot, ScanOptions, ScanSession, ScanStatus};
pub use treemap::{TRect, Tile, TreemapOptions};
