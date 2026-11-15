//! Windows'a ozgu sistem cagrilari: surucu listesi ve geri donusum kutusu.

use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use windows_sys::Win32::Storage::FileSystem::{
    GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDriveStringsW, GetVolumeInformationW,
};
use windows_sys::Win32::UI::Shell::{
    FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, SHFILEOPSTRUCTW,
    SHFileOperationW,
};

const DRIVE_REMOVABLE: u32 = 2;
const DRIVE_FIXED: u32 = 3;
const DRIVE_REMOTE: u32 = 4;
const DRIVE_CDROM: u32 = 5;
const DRIVE_RAMDISK: u32 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriveKind {
    Fixed,
    Removable,
    Remote,
    Optical,
    RamDisk,
    Other,
}

impl DriveKind {
    pub fn label_tr(self) -> &'static str {
        match self {
            DriveKind::Fixed => "Yerel disk",
            DriveKind::Removable => "Cikarilabilir",
            DriveKind::Remote => "Ag surucusu",
            DriveKind::Optical => "Optik",
            DriveKind::RamDisk => "RAM disk",
            DriveKind::Other => "Surucu",
        }
    }
}

#[derive(Clone, Debug)]
pub struct DriveInfo {
    /// `C:`
    pub letter: String,
    /// `C:\`
    pub root: String,
    pub label: String,
    pub fs: String,
    pub total: u64,
    pub free: u64,
    pub kind: DriveKind,
}

impl DriveInfo {
    pub fn used(&self) -> u64 {
        self.total.saturating_sub(self.free)
    }

    pub fn used_fraction(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            self.used() as f32 / self.total as f32
        }
    }
}

fn wide(s: &str) -> Vec<u16> {
    OsString::from(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    OsString::from_wide(&buf[..end])
        .to_string_lossy()
        .into_owned()
}

/// Tum erisilebilir suruculeri listeler (sabit, cikarilabilir, ag).
pub fn drives() -> Vec<DriveInfo> {
    let mut out = Vec::new();
    let mut buf = vec![0u16; 2048];
    let len = unsafe { GetLogicalDriveStringsW(buf.len() as u32, buf.as_mut_ptr()) } as usize;
    if len == 0 {
        return out;
    }
    let mut start = 0usize;
    while start < len.min(buf.len()) {
        let mut end = start;
        while end < buf.len() && buf[end] != 0 {
            end += 1;
        }
        if end == start {
            break;
        }
        let root = from_wide(&buf[start..end]);
        start = end + 1;
        if root.len() < 3 {
            continue;
        }
        let wide_root = wide(&root);
        let kind = match unsafe { GetDriveTypeW(wide_root.as_ptr()) } {
            DRIVE_FIXED => DriveKind::Fixed,
            DRIVE_REMOVABLE => DriveKind::Removable,
            DRIVE_REMOTE => DriveKind::Remote,
            DRIVE_CDROM => DriveKind::Optical,
            DRIVE_RAMDISK => DriveKind::RamDisk,
            _ => DriveKind::Other,
        };

        let mut free_for_caller = 0u64;
        let mut total = 0u64;
        let mut free = 0u64;
        let ok = unsafe {
            GetDiskFreeSpaceExW(
                wide_root.as_ptr(),
                &mut free_for_caller,
                &mut total,
                &mut free,
            )
        };
        if ok == 0 || total == 0 {
            // Bos optik surucu gibi medyasiz birimler atlanir.
            continue;
        }

        let mut label_buf = [0u16; 261];
        let mut fs_buf = [0u16; 65];
        let mut serial = 0u32;
        let mut max_component = 0u32;
        let mut flags = 0u32;
        let ok = unsafe {
            GetVolumeInformationW(
                wide_root.as_ptr(),
                label_buf.as_mut_ptr(),
                label_buf.len() as u32,
                &mut serial,
                &mut max_component,
                &mut flags,
                fs_buf.as_mut_ptr(),
                fs_buf.len() as u32,
            )
        };
        let label = if ok != 0 {
            from_wide(&label_buf)
        } else {
            String::new()
        };
        let fs = if ok != 0 {
            from_wide(&fs_buf)
        } else {
            String::new()
        };

        out.push(DriveInfo {
            letter: root.trim_end_matches('\\').to_string(),
            root,
            label,
            fs,
            total,
            free,
            kind,
        });
    }
    out.sort_by(|a, b| a.letter.cmp(&b.letter));
    out
}

/// Verilen yolun surucu koku (`C:\`). UNC yollarinda `None`.
pub fn volume_root(path: &str) -> Option<String> {
    let bytes = path.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':' {
        Some(format!("{}:\\", path.chars().next()?.to_ascii_uppercase()))
    } else {
        None
    }
}

/// Dosya/klasorleri Geri Donusum Kutusu'na tasir. Kalici silmez.
pub fn recycle(paths: &[String]) -> Result<(), String> {
    if paths.is_empty() {
        return Ok(());
    }
    let mut from: Vec<u16> = Vec::new();
    for p in paths {
        from.extend(OsString::from(p).encode_wide());
        from.push(0);
    }
    from.push(0); // cift null sonlandirma

    let flags = FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI;
    let mut op: SHFILEOPSTRUCTW = unsafe { std::mem::zeroed() };
    op.wFunc = FO_DELETE;
    op.pFrom = from.as_ptr();
    op.fFlags = flags as u16;

    let result = unsafe { SHFileOperationW(&mut op) };
    if result != 0 {
        return Err(match result {
            0x78 => "Erisim reddedildi (dosya kilitli veya yetki yok).".to_string(),
            0x79 => "Hedef klasore erisilemedi.".to_string(),
            0x7C => "Gecersiz dosya yolu.".to_string(),
            1223 => "Islem iptal edildi.".to_string(),
            code => format!("Geri donusume tasinamadi (kod 0x{code:X})."),
        });
    }
    if op.fAnyOperationsAborted != 0 {
        return Err("Islem iptal edildi.".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_root_is_detected() {
        assert_eq!(volume_root("C:\\Users\\Kayon").as_deref(), Some("C:\\"));
        assert_eq!(volume_root("d:\\veri").as_deref(), Some("D:\\"));
        assert_eq!(volume_root(r"\\sunucu\pay"), None);
    }

    #[test]
    fn drive_list_contains_c() {
        let list = drives();
        assert!(list.iter().any(|d| d.letter.eq_ignore_ascii_case("C:")));
    }

    /// Gercek Geri Donusum Kutusu cagrisini dogrular; varsayilan olarak calismaz.
    #[test]
    #[ignore = "Geri Donusum Kutusu'na gercek dosya tasir"]
    fn recycle_moves_file_to_bin() {
        let path = std::env::temp_dir().join(format!(
            "iskandil-recycle-{}-{}.txt",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, b"Iskandil geri donusum testi").unwrap();
        let text = path.to_string_lossy().to_string();
        let result = recycle(&[text]);
        assert!(result.is_ok(), "{result:?}");
        assert!(!path.exists());
    }
}
