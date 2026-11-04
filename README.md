# SkadooshNotch — Dynamic Island / Notch untuk Windows

Aplikasi notch modern, ringan (~3 MB, RAM ~30 MB), dan kaya fitur khusus Windows 10 & 11.

## Fitur Utama
1. **Jadwal Sholat & Adzan:** Auto-detect lokasi GPS/IP, hitung waktu sholat (Kemenag RI), alarm file adzan custom, hitung mundur sholat berikutnya.
2. **Universal Media Player + Lirik:** Sinkronisasi Spotify, YouTube, WMP, Apple Music via Windows GSMTC + fetch lirik otomatis via LRCLIB API.
3. **Smart Auto-Shrink & Auto-Hide:** Notch mengecil jika jendela browser aktif (agar tab browser tidak tertutup), serta otomatis sembunyi saat game/video fullscreen.
4. **Power & Screen Timeout:** Slider atur waktu layar mati & sleep PC dalam menit, tombol Lock, Sleep, dan Screen Off instan.
5. **Screenshot & Record (`Ctrl+Shift+R`):** Snip area tertentu, tangkap layar penuh, atau rekam layar.
6. **Produktivitas Ringan:**
   - Mini Calendar
   - To-Do List dengan tanggal deadline
   - Quick Notes auto-save
   - Clipboard Manager (riwayat copy)
   - File Shelf (drop file sementara)

---

## Cara Build Jadi `.exe` Otomatis (Tanpa Install SDK di PC)

1. Buat repository baru di [GitHub](https://github.com/new) (nama bebas, misal `winnotch`).
2. Upload atau push semua file project ini ke repo tersebut:
   ```bash
   git init
   git add .
   git commit -m "Initial commit SkadooshNotch"
   git branch -M main
   git remote add origin https://github.com/USERNAME/winnotch.git
   git push -u origin main
   ```
3. Buka tab **Actions** di repo GitHub Anda.
4. Tunggu workflow **Build SkadooshNotch EXE** selesai (~1–2 menit).
5. Klik build yang selesai, lalu scroll ke bawah ke bagian **Artifacts** dan download `SkadooshNotch-exe.zip`.
6. Ekstrak dan jalankan `SkadooshNotch.exe` langsung di PC Windows!

---

## Build Manual di PC (Jika Punya .NET SDK 8)
```powershell
dotnet publish -c Release -r win-x64 --self-contained false -p:PublishSingleFile=true -o publish/
```
Output `.exe` ada di folder `publish/SkadooshNotch.exe`.
