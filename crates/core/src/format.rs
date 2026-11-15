//! Turkce sayi, boyut, sure ve hiz bicimlendirme.
//!
//! Windows'un Turkce yerel ayarlarina uygun: binlik ayirici `.`, ondalik `,`.

use std::time::Duration;

/// Binlik ayiricili tam sayi: `1.284.032`.
pub fn count(n: u64) -> String {
    let raw = n.to_string();
    let mut out = String::with_capacity(raw.len() + raw.len() / 3);
    for (i, ch) in raw.chars().enumerate() {
        if i > 0 && (raw.len() - i) % 3 == 0 {
            out.push('.');
        }
        out.push(ch);
    }
    out
}

/// Ondalik degeri Turkce virgul ile yazar: `412,8`.
pub fn decimal1(v: f64) -> String {
    let s = format!("{v:.1}");
    s.replace('.', ",")
}

/// Bayt boyutunu 1024 tabaninda kisa gosterir: `412,8 GB`, `781 KB`, `12 B`.
pub fn bytes(n: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
    let mut value = n as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{n} B")
    } else if value >= 100.0 {
        format!("{:.0} {}", value, UNITS[unit])
    } else {
        format!("{:.1} {}", value, UNITS[unit]).replace('.', ",")
    }
}

/// Tam boyut, birim adi: `412,8 GB`.
pub fn bytes_long(n: u64) -> String {
    bytes(n)
}

/// Yuzde: `%23,4`.
pub fn percent(part: u64, total: u64) -> String {
    if total == 0 {
        return "%0,0".to_string();
    }
    format!("%{:.1}", part as f64 * 100.0 / total as f64).replace('.', ",")
}

/// Yuzde, sayi olarak.
pub fn fraction(part: u64, total: u64) -> f32 {
    if total == 0 {
        0.0
    } else {
        (part as f64 / total as f64) as f32
    }
}

/// Sure: `0,8 sn`, `12,4 sn`, `2 dk 14 sn`.
pub fn duration(d: Duration) -> String {
    let secs = d.as_secs_f64();
    if secs < 60.0 {
        format!("{secs:.1} sn").replace('.', ",")
    } else {
        let m = (secs / 60.0).floor() as u64;
        let s = (secs - m as f64 * 60.0).round() as u64;
        format!("{m} dk {s} sn")
    }
}

/// Tarama hizi: `284.512 dosya/sn` veya `412,3 MB/sn`.
pub fn rate_count(files: u64, d: Duration) -> String {
    let secs = d.as_secs_f64().max(0.001);
    format!("{}/sn", count((files as f64 / secs).round() as u64))
}

pub fn rate_bytes(n: u64, d: Duration) -> String {
    let secs = d.as_secs_f64().max(0.001);
    format!("{}/sn", bytes((n as f64 / secs) as u64))
}

/// Tarih: `2026-11-14 09:41`. Unix saniyesi 0 ise `-`.
pub fn timestamp(unix: i64) -> String {
    if unix <= 0 {
        return "-".to_string();
    }
    // Gun hesabi: sivil tarih algoritmasi (Howard Hinnant).
    let days = unix.div_euclid(86_400);
    let secs = unix.rem_euclid(86_400);
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}",
        secs / 3600,
        (secs % 3600) / 60
    )
}

fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Turkce/Unicode duyarli arama icin kucuk harfe indirger (I/İ/ı -> i).
pub fn fold_tr(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            'I' | 'İ' | 'ı' => out.push('i'),
            _ => out.extend(ch.to_lowercase()),
        }
    }
    out
}

/// `haystack` icinde `folded_needle` (fold_tr ile uretilmis) aramasi.
///
/// Cogunlukla ayirma yapmadan, yigin tamponuyla calisir; 128 karakterden uzun
/// adlarda (nadir) allocate eden yedek yola duser.
pub fn contains_folded(haystack: &str, folded_needle: &str) -> bool {
    if folded_needle.is_empty() {
        return true;
    }
    let needle: Vec<char> = folded_needle.chars().collect();
    let mut buf = ['\0'; 128];
    let mut len = 0usize;
    for ch in haystack.chars() {
        if !push_folded(ch, &mut buf, &mut len) {
            return contains_folded_slow(haystack, &needle);
        }
    }
    if needle.len() > len {
        return false;
    }
    let span = needle.len();
    for start in 0..=len - span {
        if buf[start..start + span] == needle[..] {
            return true;
        }
    }
    false
}

fn push_folded(ch: char, buf: &mut [char; 128], len: &mut usize) -> bool {
    match ch {
        'I' | 'İ' | 'ı' => {
            if *len >= buf.len() {
                return false;
            }
            buf[*len] = 'i';
            *len += 1;
        }
        _ => {
            for lower in ch.to_lowercase() {
                if *len >= buf.len() {
                    return false;
                }
                buf[*len] = lower;
                *len += 1;
            }
        }
    }
    true
}

fn contains_folded_slow(haystack: &str, needle: &[char]) -> bool {
    let folded = fold_tr(haystack);
    let folded: Vec<char> = folded.chars().collect();
    if needle.len() > folded.len() {
        return false;
    }
    folded.windows(needle.len()).any(|window| window == needle)
}

/// Uzun yolu orta yerinden kisaltir: `C:\Users\...\node_modules`.
pub fn middle_ellipsis(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= max_chars || max_chars < 8 {
        return s.to_string();
    }
    let keep_front = (max_chars - 3) / 2;
    let keep_back = max_chars - 3 - keep_front;
    let mut out = String::with_capacity(max_chars);
    out.extend(&chars[..keep_front]);
    out.push_str("...");
    out.extend(&chars[chars.len() - keep_back..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thousands_use_dots() {
        assert_eq!(count(0), "0");
        assert_eq!(count(999), "999");
        assert_eq!(count(1_284_032), "1.284.032");
    }

    #[test]
    fn bytes_use_binary_units_and_comma() {
        assert_eq!(bytes(0), "0 B");
        assert_eq!(bytes(900), "900 B");
        assert_eq!(bytes(1024), "1,0 KB");
        assert_eq!(bytes(1536), "1,5 KB");
        assert_eq!(bytes(1024 * 1024 * 1024), "1,0 GB");
    }

    #[test]
    fn percent_is_turkish() {
        assert_eq!(percent(1, 4), "%25,0");
        assert_eq!(percent(0, 0), "%0,0");
    }

    #[test]
    fn timestamp_formats() {
        assert_eq!(timestamp(0), "-");
        // 2026-11-14 00:00:00 UTC
        assert_eq!(timestamp(1_794_614_400), "2026-11-14 00:00");
    }

    #[test]
    fn turkish_fold_matches_dotted_and_dotless_i() {
        assert!(contains_folded("İndirilenler", "indir"));
        assert!(contains_folded("INDIR", "indir"));
        assert!(contains_folded("ısı", "isi"));
        assert!(!contains_folded("belgeler", "resim"));
    }

    #[test]
    fn fold_search_handles_overlaps_and_long_names() {
        assert!(contains_folded("aaab", "aab"));
        assert!(contains_folded("İİİİİİİİİİİİİ", "iii"));
        let long_ascii = "x".repeat(200) + "BUL";
        assert!(contains_folded(&long_ascii, "bul"));
        let long_turkish = format!("{}İÇERİK", "ş".repeat(140));
        assert!(contains_folded(&long_turkish, "içerik"));
    }

    #[test]
    fn middle_ellipsis_keeps_ends() {
        assert_eq!(middle_ellipsis("kisa", 10), "kisa");
        let short = middle_ellipsis("C:\\Users\\Kayon\\AppData\\Local\\Temp", 20);
        assert!(short.chars().count() <= 20);
        assert!(short.starts_with("C:\\Us"));
        assert!(short.ends_with("Temp"));
    }
}
