"""Iskandil icin ornek agac uretir.

Dosyalar NTFS sparse olarak olusturulur: mantiksal boyutlari buyuk, fiziksel
yer kaplamalari neredeyse sifirdir. Yalnizca gelistirme ve ekran goruntusu
icin kullanilir; gercek disk kullanimini olcmek icin kullanilmaz.
"""

from __future__ import annotations

import ctypes
import os
import sys
from pathlib import Path

GENERIC_WRITE = 0x40000000
FILE_SHARE_READ = 1
OPEN_ALWAYS = 4
FSCTL_SET_SPARSE = 0x000900C4

kernel32 = ctypes.windll.kernel32
kernel32.CreateFileW.restype = ctypes.c_void_p
kernel32.SetFilePointerEx.argtypes = [
    ctypes.c_void_p,
    ctypes.c_longlong,
    ctypes.c_void_p,
    ctypes.c_uint32,
]
kernel32.SetEndOfFile.argtypes = [ctypes.c_void_p]
kernel32.DeviceIoControl.argtypes = [
    ctypes.c_void_p,
    ctypes.c_uint32,
    ctypes.c_void_p,
    ctypes.c_uint32,
    ctypes.c_void_p,
    ctypes.c_uint32,
    ctypes.POINTER(ctypes.c_uint32),
    ctypes.c_void_p,
]
kernel32.CloseHandle.argtypes = [ctypes.c_void_p]


def sparse_file(path: Path, size: int) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    handle = kernel32.CreateFileW(
        str(path), GENERIC_WRITE, FILE_SHARE_READ, None, OPEN_ALWAYS, 0, None
    )
    if handle in (None, ctypes.c_void_p(-1).value):
        raise OSError(f"CreateFileW failed: {path}")
    try:
        returned = ctypes.c_uint32()
        kernel32.DeviceIoControl(
            handle, FSCTL_SET_SPARSE, None, 0, None, 0, ctypes.byref(returned), None
        )
        kernel32.SetFilePointerEx(handle, size, None, 0)
        if not kernel32.SetEndOfFile(handle):
            raise OSError(f"SetEndOfFile failed: {path}")
    finally:
        kernel32.CloseHandle(handle)


MB = 1024 * 1024
KB = 1024


def build(root: Path) -> None:
    files: list[tuple[str, int]] = [
        # Film arsivi
        ("Film Arşivi/2026 Yaz Kaydı.mp4", 1274 * MB),
        ("Film Arşivi/Belgesel - Boğazın İki Yakası.mkv", 892 * MB),
        ("Film Arşivi/kısa klip - vapur.mp4", 118 * MB),
        ("Film Arşivi/çekim notları.txt", 6 * KB),
        # Fotograflar
        ("Fotoğraflar/İstanbul 2026/DSC_0142.jpg", 18 * MB),
        ("Fotoğraflar/İstanbul 2026/DSC_0143.jpg", 21 * MB),
        ("Fotoğraflar/İstanbul 2026/DSC_0144.jpg", 19 * MB),
        ("Fotoğraflar/İstanbul 2026/DSC_0145.jpg", 17 * MB),
        ("Fotoğraflar/İstanbul 2026/DSC_0146.jpg", 23 * MB),
        ("Fotoğraflar/İstanbul 2026/DSC_0147.jpg", 16 * MB),
        ("Fotoğraflar/Kapadokya/IMG_2201.jpg", 24 * MB),
        ("Fotoğraflar/Kapadokya/IMG_2202.jpg", 26 * MB),
        ("Fotoğraflar/Kapadokya/IMG_2203.jpg", 22 * MB),
        ("Fotoğraflar/Kapadokya/IMG_2204.jpg", 25 * MB),
        ("Fotoğraflar/arşiv-fotoğraflar.zip", 410 * MB),
        # Projeler
        ("Projeler/iskandil/src/main.rs", 42 * KB),
        ("Projeler/iskandil/src/scanner.rs", 18 * KB),
        ("Projeler/iskandil/src/ui.rs", 96 * KB),
        ("Projeler/iskandil/src/theme.rs", 22 * KB),
        ("Projeler/iskandil/target/debug/iskandil.exe", 18 * MB),
        ("Projeler/iskandil/target/debug/deps/libegui.rlib", 62 * MB),
        ("Projeler/iskandil/target/debug/deps/libwgpu.rlib", 148 * MB),
        ("Projeler/iskandil/target/debug/deps/libwindows_sys.rlib", 96 * MB),
        ("Projeler/iskandil/target/debug/build/obj.o", 240 * MB),
        ("Projeler/iskandil/Cargo.lock", 74 * KB),
        ("Projeler/website/package.json", 2 * KB),
        ("Projeler/website/node_modules/react/index.js", 8 * MB),
        ("Projeler/website/node_modules/three/three.module.js", 24 * MB),
        ("Projeler/website/node_modules/typescript/lib/tsc.js", 68 * MB),
        ("Projeler/website/node_modules/vite/dist/node/chunks/dep.js", 31 * MB),
        ("Projeler/website/dist/app.js", 1200 * KB),
        ("Projeler/website/dist/styles.css", 180 * KB),
        ("Projeler/website/dist/assets/hero.webp", 24 * MB),
        ("Projeler/yedek-2025-12.zip", 320 * MB),
        # Muzik
        ("Müzik/Radyo Kayıtları/gece programı 01.mp3", 45 * MB),
        ("Müzik/Radyo Kayıtları/gece programı 02.mp3", 47 * MB),
        ("Müzik/Radyo Kayıtları/gece programı 03.mp3", 44 * MB),
        ("Müzik/Radyo Kayıtları/gece programı 04.mp3", 46 * MB),
        ("Müzik/albüm/01 açılış.flac", 38 * MB),
        ("Müzik/albüm/02 boğaz.flac", 42 * MB),
        ("Müzik/albüm/03 martılar.flac", 39 * MB),
        ("Müzik/albüm/04 akşam.flac", 41 * MB),
        ("Müzik/albüm/05 dönüş.flac", 37 * MB),
        # Belgeler
        ("Belgeler/sunum.pptx", 48 * MB),
        ("Belgeler/rapor.pdf", 12 * MB),
        ("Belgeler/bütçe.xlsx", 4 * MB),
        ("Belgeler/notlar.md", 12 * KB),
        ("Belgeler/sözleşmeler/2026-01.pdf", 2 * MB),
        ("Belgeler/sözleşmeler/2026-02.pdf", 2 * MB),
        ("Belgeler/sözleşmeler/2026-03.pdf", 2 * MB),
        # Yedekler
        ("Yedekler/sistem-yedek.7z", 1430 * MB),
        ("Yedekler/eski-proje.rar", 210 * MB),
        # Kurulum
        ("kurulum/Windows11.iso", 640 * MB),
        ("kurulum/setup.exe", 84 * MB),
        ("kurulum/notes.txt", 3 * KB),
        ("kurulum/bos-dosya.dat", 0),
    ]
    for rel, size in files:
        sparse_file(root / rel, size)
    keep = root / "Projeler/website/.gitkeep"
    keep.parent.mkdir(parents=True, exist_ok=True)
    keep.write_text("", encoding="utf-8")
    total = sum(size for _, size in files)
    print(f"{len(files)} dosya, {total / (1024**3):.2f} GB mantiksal boyut")


if __name__ == "__main__":
    target = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).parent.parent / "İskandil Demo"
    build(target)
    print(f"hazir: {target}")
