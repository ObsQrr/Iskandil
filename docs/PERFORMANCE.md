# Performans

Ölçüm makinesi: Ryzen 5 5500 (6C/12T), 16 GB RAM, RTX 3060, NVMe SSD,
Windows 11 25H2. Ölçümler `iskandil --bench <yol>` ile yapılır; arayüz açılmaz.
Release derlemesi statik CRT ile alınmıştır.

## Tarama

| Derleme | Hedef | Dosya | Klasör | Boyut | Süre | Hız | Hata |
|---|---|---:|---:|---:|---:|---:|---:|
| release (soğuk) | `C:\Windows\System32` | 24.301 | 1.631 | 8,98 GB | 0,551 sn | 44.128 dosya/sn | 18 |
| release (sıcak) | `C:\Windows\System32` | 24.301 | 1.631 | 8,98 GB | 0,261 sn | 93.013 dosya/sn | 18 |
| release (soğuk) | `C:\Users\Kayon` | 1.491.433 | 278.425 | 236,4 GB | 31,683 sn | 47.073 dosya/sn | 896 |
| release (sıcak) | `C:\Users\Kayon` | 1.492.156 | 278.648 | 237,0 GB | 18,527 sn | 80.541 dosya/sn | 896 |
| debug | `C:\Users\Kayon` | 1.488.350 | 278.006 | 234,7 GB | 20,329 sn | 73.213 dosya/sn | 896 |

- 1,49 milyon girişlik taramada **tepe çalışma kümesi 196,5 MB** (200 ms
  aralıklarla `WorkingSet64` örneklenerek ölçüldü).
- Debug ve release arasındaki fark, ölçümlerin farklı disk önbelleği
  durumlarında alınmasından kaynaklanır; sıcak/soğuk farkı algoritmadan
  bağımsızdır. Karşılaştırma için aynı hedefi iki kez koşun.
- Örnek demo ağacı (57 dosya, 6,6 GB mantıksal) release'te ~0,03 sn.

## Notlar

- Tarama NTFS MFT'sini okumaz; paralel dizin taraması yapar. Yönetici hakkı
  istemez, ağ ve çıkarılabilir birimlerde de çalışır.
- Windows `DirEntry` metadata'sı `FindFirstFile` verisinden geldiği için dosya
  başına ek sistem çağrısı yapılmaz.
- Hatalar çoğunlukla korumalı sistem klasörleridir (`System Volume Information`,
  bazı AppData alt klasörleri); tarama durmaz, sayı status çubuğunda görünür.
- Sparse demo ağacı (`scripts/make_demo.py`) 6,6 GB mantıksal boyutu sıfır
  fiziksel alanla üretir (FSCTL_SET_SPARSE + SetEndOfFile); yalnızca görsel
  test içindir. `fsutil file createnew` ve seek+write bu makinede alan ayırır.

## Arayüz

- Treemap 4.000 döşeme ve klasör başına 240 çocukla sınırlıdır.
- Tür dağılımı ve en büyük dosyalar görünüm kökü değişince bir kez hesaplanır
  (1,5M girişte ~30–60 ms).
- Tree satırları sanallaştırılmıştır (`ScrollArea::show_rows`).
- Arama, 128 karaktere kadar adları yığın tamponuyla katlar; yalnızca uzun
  adlarda allocate eder.
