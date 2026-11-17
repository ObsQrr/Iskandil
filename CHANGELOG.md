# Değişiklik Günlüğü

## [1.2.0] — 2026-11-17

- Dosya ağacına ve arama sonuçlarına boyut, ad ve değiştirilme tarihine göre
  artan ve azalan sıralama eklendi. Klasör ağacında sıralama her klasörün
  kendi çocuklarına uygulanır.
- Sıralama seçenekleri ve aramada sonuç sınırı için regresyon testi eklendi.

## [0.1.0] — 2026-11-15

İlk sürüm: Rust/egui tabanlı Windows disk alanı gezgini.

### Eklendi

- Paralel dizin taraması (12 iş parçacığı, 2–16 arası otomatik), `\\?\` uzun yol
  desteği, yeniden ayırma noktalarında döngü koruması, canlı ilerleme ve iptal
  (o ana kadarki sonuç kısmi olarak korunur).
- Arena tabanlı dosya ağacı; boyut/dosya/klasör toplamları, boyuta göre sıralı
  sanallaştırılmış ağaç görünümü, satır içi pay çubukları.
- Squarified treemap: iç içe klasör başlık şeritleri, tür renkleri, ipucu kartı,
  tıklama/çift tıklama, sağ tık menüsü, 4.000 döşeme sınırı.
- Türkçe arama (I/İ/ı katlaması, yığın tamponlu), tür dağılımı, seçili klasörde
  en büyük 5 dosya, breadcrumb ve “Yukarı”.
- Gezgin'de göster, yolu kopyala, Geri Dönüşüm Kutusu'na taşı (onay modalı;
  kalıcı silme yok).
- Açık “grafik kâğıdı” ve koyu “gece ölçümü” temaları; Bahnschrift + Cascadia
  Mono + Segoe UI Variable; 18 px sol derinlik cetveli.
- CLI: `--scan`, `--view`, `--search`, `--bench`, `--shot`, `--shot-now`,
  `--ask-delete-largest`, `--theme`, `--size`.
- Statik CRT release EXE, gömülü ICO ve sürüm bilgisi, portable zip.

### Doğrulama

- `cargo test -p iskandil-core`: **32 geçti, 1 ignored** (gerçek Geri Dönüşüm
  Kutusu testi; `--ignored` ile ayrıca çalıştırıldı).
- `cargo clippy -D warnings` ve `cargo fmt --check`: temiz.
- Yedi ekran gerçek release EXE'den alındı: `docs/screenshots/01..07`.

### Ölçüm (Ryzen 5 5500 / NVMe)

| Hedef | Dosya | Boyut | Soğuk | Sıcak |
|---|---:|---:|---:|---:|
| `C:\Windows\System32` | 24.301 | 8,98 GB | 0,551 sn | 0,261 sn |
| `C:\Users\Kayon` | ~1,49 M | ~237 GB | 31,683 sn | 18,527 sn |

1,49 milyon girişte tepe çalışma kümesi **196,5 MB**.

### Yayın

- `iskandil.exe` — **14.873.600 bayt**, SHA-256
  `43DCC4BA8C3D7DAE8D6CABA245F24181B389AB5FA361E74C3DFAC2C5030DBA74`
- `iskandil-0.1.0-win-x64.zip` — **6.071.462 bayt**, SHA-256
  `A95C4CCCAB64A9D133A95C29AD76582D636D65BCC27EF2D673E55C9EE742FCCA`

### Bilinen sınırlar

- NTFS MFT/USN hızlı modu yok; hard link'ler çift sayılabilir; mantıksal boyut.
- CSV/PDF/JSON dışa aktarma, yinelenen dosya bulucu, snapshot/karşılaştırma,
  zamanlanmış tarama, installer ve kod imzalama yok.
- Özel çizilen ağaç/treemap düğmeleri için tam erişilebilirlik semantiği yok;
  klavye ile satır gezinmesi yok (kısayollar var).
- Fiziksel çoklu monitör/DPI testi yapılmadı.
