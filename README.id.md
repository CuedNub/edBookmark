
---

# edbookmark

Aplikasi TUI (Terminal User Interface) untuk mengelola bookmark, dibangun dengan Rust dan Ratatui. Dirancang khusus untuk Arch Linux + Hyprland (Wayland) dengan Omarchy.

## Fitur

- **Navigasi keyboard vim-like** — `j/k` untuk navigasi, `/` untuk search, `a` untuk add, `e` untuk edit, `d` untuk delete
- **Fuzzy search multi-kata** — ketik `pano rama` untuk mencari bookmark yang mengandung kedua kata tersebut
- **Cursor-aware text editing** — gerakkan kursor dengan `← →`, `Home/End`, `Ctrl+A/E` di form dan search bar
- **Import bookmark** — dari Chromium (JSON) dan Firefox (HTML export)
- **Export bookmark** — ke format JSON dan HTML (kompatibel dengan Chrome/Firefox)
- **Frameless Chromium webapp** — bookmark dibuka sebagai window Chromium tanpa frame via `systemd-run`
- **Tema Ayu yang bisa dikustomisasi** — semua warna bisa diubah melalui file konfigurasi
- **Background transparan** — mengikuti background terminal Anda
- **Border warna-warni** — setiap sisi border bisa memiliki warna berbeda
- **Integrasi Walker** — bisa dibuka dari launcher Walker sebagai window floating
- **Clipboard** — salin URL ke clipboard dengan `y` (menggunakan `wl-copy`)
- **Multi-select** — pilih beberapa bookmark sekaligus dengan `Space`, hapus sekaligus dengan `D`

## Screenshot

> *Akan ditambahkan*

## Persyaratan Sistem

### Teruji Pada

| Komponen | Versi |
|---|---|
| OS | Arch Linux (Omarchy) |
| Window Manager | Hyprland |
| Protokol Display | Wayland |
| Terminal | Kitty |
| Shell | Bash |
| Browser | Chromium |

### Dependensi Wajib

| Paket | Kegunaan |
|---|---|
| `rust` | Compiler dan Cargo (build dari source) |
| `kitty` | Terminal emulator untuk menjalankan TUI |
| `chromium` | Browser untuk membuka bookmark |
| `wl-clipboard` | Copy URL ke clipboard (`wl-copy`) |
| `xdg-utils` | Deteksi browser default (`xdg-settings`) |
| `util-linux` | Detach proses (`setsid`) |
| `systemd` | Menjalankan browser sebagai service terpisah (`systemd-run`) |

### Dependensi Opsional

| Paket | Kegunaan |
|---|---|
| `uwsm-app` | Alternatif launcher untuk Wayland session |
| `walker` | Application launcher untuk integrasi desktop |
| `hyprland` | Reload otomatis window rules (`hyprctl`) |

## Instalasi

### Clone dan Install

```bash
git clone git@github.com:CuedNub/edBookmark.git
cd edBookmark
bash install.sh
```

Installer akan:
1. Mengecek semua persyaratan sistem
2. Menawarkan instalasi otomatis paket yang belum terpasang (via `pacman`)
3. Build binary release yang teroptimasi
4. Meminta izin sebelum menulis setiap file
5. Mengkonfigurasi Hyprland window rules
6. Memverifikasi instalasi

### Lokasi File Setelah Install

| File | Lokasi |
|---|---|
| Binary | `~/.local/bin/edbookmark` |
| Walker wrapper | `~/.local/bin/edbookmark-walker` |
| Konfigurasi | `~/.config/edbookmark/config.toml` |
| Data bookmark | `~/.local/share/edbookmark/bookmarks.json` |
| Desktop entry | `~/.local/share/applications/edbookmark.desktop` |
| Log launcher | `~/.local/state/edbookmark/launcher.log` |

## Uninstall

```bash
cd edBookmark
bash uninstall-edbookmark.sh
```

Uninstaller akan:
1. Mendeteksi semua komponen yang terpasang
2. Meminta izin sebelum menghapus setiap item
3. Menampilkan konfirmasi final sebelum eksekusi
4. Membuat backup file penting sebelum menghapus

### Reinstall

Jika terjadi masalah, cukup uninstall lalu install ulang:

```bash
bash uninstall-edbookmark.sh
bash install.sh
```

## Penggunaan

### Membuka Aplikasi

```bash
# Dari terminal
edbookmark

# Dari Walker (application launcher)
# Ketik "edbookmark" → Enter
```

### Alur Penggunaan

```
Walker → "edbookmark" → Enter
    ↓
Kitty terbuka (floating 800x500, centered)
    ↓
Pilih bookmark dengan j/k atau /search
    ↓
Enter → edbookmark tertutup + Chromium frameless terbuka
```

### Import dan Export

```bash
# Import dari Chromium
edbookmark --import chromium

# Import dari file HTML (Firefox export)
edbookmark --import-file bookmarks.html

# Export ke JSON
edbookmark --export json -o backup.json

# Export ke HTML (bisa di-import ke Chrome/Firefox)
edbookmark --export html -o bookmarks.html
```

## Keybinding

### Normal Mode

| Key | Aksi |
|---|---|
| `j` / `↓` | Pindah ke bawah |
| `k` / `↑` | Pindah ke atas |
| `g` | Ke bookmark paling atas |
| `G` | Ke bookmark paling bawah |
| `Enter` | Buka bookmark di Chromium (frameless) |
| `/` | Masuk mode pencarian |
| `a` | Tambah bookmark baru |
| `e` | Edit bookmark yang dipilih |
| `d` | Hapus bookmark yang dipilih |
| `Space` | Toggle pilih (multi-select) |
| `D` | Hapus semua yang dipilih |
| `y` | Salin URL ke clipboard |
| `?` | Tampilkan bantuan keybinding |
| `q` / `Esc` | Keluar |

### Search Mode

| Key | Aksi |
|---|---|
| *Karakter apapun* | Ketik pencarian |
| `Enter` | Konfirmasi pencarian |
| `Esc` | Batalkan pencarian |
| `↓` / `Ctrl+N` | Navigasi hasil ke bawah |
| `↑` / `Ctrl+P` | Navigasi hasil ke atas |
| `←` / `Ctrl+B` | Kursor ke kiri |
| `→` / `Ctrl+F` | Kursor ke kanan |
| `Home` / `Ctrl+A` | Kursor ke awal |
| `End` / `Ctrl+E` | Kursor ke akhir |
| `Delete` | Hapus karakter di kursor |
| `Backspace` | Hapus karakter sebelum kursor |
| `Ctrl+W` | Hapus satu kata |
| `Ctrl+U` | Hapus seluruh input |

### Form Mode (Add / Edit)

| Key | Aksi |
|---|---|
| *Karakter apapun* | Ketik di field aktif |
| `Tab` | Pindah ke field berikutnya |
| `Shift+Tab` | Pindah ke field sebelumnya |
| `Ctrl+S` | Simpan |
| `Esc` | Batalkan |
| `←` / `Ctrl+B` | Kursor ke kiri |
| `→` / `Ctrl+F` | Kursor ke kanan |
| `Home` / `Ctrl+A` | Kursor ke awal |
| `End` / `Ctrl+E` | Kursor ke akhir |
| `Delete` | Hapus karakter di kursor |
| `Backspace` | Hapus karakter sebelum kursor |
| `Ctrl+W` | Hapus satu kata |
| `Ctrl+U` | Hapus seluruh field |

### Delete Confirm

| Key | Aksi |
|---|---|
| `y` / `Enter` | Konfirmasi hapus |
| `n` / `Esc` | Batalkan |

## Konfigurasi

File konfigurasi terletak di `~/.config/edbookmark/config.toml`.

### Contoh Konfigurasi

```toml
[window]
width = 100
height = 30

[launcher]
command = "omarchy-launch-webapp"
args = ["--isolate"]

[paths]
bookmarks = "~/.local/share/edbookmark/bookmarks.json"

[theme]
preset = "ayu-dark"
transparent_bg = true

[theme.colors]
# Background & foreground
bg = "reset"           # "reset" = transparan (ikut terminal)
fg = "#BFBDB6"

# Komponen bookmark
name = "#E6E1CF"       # Warna nama bookmark
url = "#95E6CB"        # Warna URL
folder = "#D2A6FF"     # Warna folder

# Header tabel
header = "#FF8F40"
header_bg = "reset"

# Baris yang dipilih
selected_fg = "#E6E1CF"
selected_bg = "#2D4F67"

# Multi-select
multiselect_fg = "#E6E1CF"
multiselect_bg = "#3E4B59"

# Search bar
search_fg = "#E6E1CF"
search_border = "#39BAE6"
match_highlight = "#F07178"

# Status bar
status_fg = "#AAD94C"
status_bg = "reset"

# Aksen dan muted
accent = "#E6B450"
muted = "#565B66"

# Border warna-warni (4 sisi)
border_top = "#39BAE6"
border_right = "#AAD94C"
border_bottom = "#FF8F40"
border_left = "#D2A6FF"

# Form field
field_active_border = "#39BAE6"
field_inactive_border = "#565B66"
field_text = "#E6E1CF"
field_placeholder = "#565B66"

# Dialog hapus
delete_border = "#F07178"
delete_text = "#F07178"

# Tombol
button_save_fg = "#0D1017"
button_save_bg = "#AAD94C"
button_cancel_fg = "#BFBDB6"
button_cancel_bg = "#3E4B59"
button_delete_fg = "#0D1017"
button_delete_bg = "#F07178"
```

### Format Warna yang Didukung

| Format | Contoh |
|---|---|
| Hex | `"#39BAE6"` |
| Named | `"red"`, `"green"`, `"blue"`, `"cyan"`, `"magenta"`, `"yellow"`, `"white"`, `"black"` |
| Transparan | `"reset"` atau `"transparent"` |

## Hyprland Window Rules

Installer otomatis menambahkan rules berikut ke `~/.config/hypr/hyprland.conf`:

```ini
# BEGIN edbookmark rules
windowrule = float on, match:title ^(edbookmark)$
windowrule = center on, match:title ^(edbookmark)$
windowrule = size 800 500, match:title ^(edbookmark)$
# END edbookmark rules
```

Ubah ukuran window dengan mengedit nilai `size 800 500` sesuai kebutuhan.

## Struktur Proyek

```
edBookmark/
├── Cargo.toml              # Konfigurasi proyek Rust
├── Cargo.lock              # Lock file dependensi
├── LICENSE                 # Lisensi MIT
├── install.sh              # Installer interaktif
├── uninstall-edbookmark.sh # Uninstaller interaktif
├── config/
│   └── default.toml        # Konfigurasi default
├── dist/
│   └── edbookmark.desktop  # Template desktop entry
├── src/
│   ├── main.rs             # Entry point + CLI
│   ├── app.rs              # State aplikasi & event loop
│   ├── bookmark.rs         # Struct & logic bookmark
│   ├── config.rs           # Parser konfigurasi
│   ├── import_export.rs    # Import/export Chrome & Firefox
│   ├── keybinding.rs       # Mode & mapping keybinding
│   ├── launcher.rs         # Eksekusi browser (systemd-run)
│   ├── search.rs           # Fuzzy search engine
│   ├── storage.rs          # Read/write file JSON
│   ├── theme.rs            # Parser tema & warna
│   └── ui/
│       ├── mod.rs
│       ├── main_view.rs    # Layout utama
│       ├── search_bar.rs   # Widget search bar
│       ├── bookmark_list.rs # Widget tabel bookmark
│       ├── form_view.rs    # Widget form add/edit
│       ├── delete_dialog.rs # Widget dialog hapus
│       ├── status_bar.rs   # Widget status bar
│       └── help_popup.rs   # Widget popup bantuan
├── data/
│   └── .gitkeep
└── assets/
    └── .gitkeep
```

## Tech Stack

| Komponen | Teknologi |
|---|---|
| Bahasa | Rust |
| TUI Framework | Ratatui + Crossterm |
| Fuzzy Search | fuzzy-matcher (Skim algorithm) |
| Serialisasi | serde + serde_json |
| Konfigurasi | toml |
| CLI | clap |
| ID | uuid v4 |
| Timestamp | chrono |

## Troubleshooting

### edbookmark tidak ditemukan setelah install

Pastikan `~/.local/bin` ada di PATH:

```bash
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Chromium tidak terbuka saat Enter

Cek log launcher:

```bash
cat ~/.local/state/edbookmark/launcher.log
```

### Chromium tidak terbuka dari Walker

Pastikan file desktop entry menggunakan walker wrapper:

```bash
cat ~/.local/share/applications/edbookmark.desktop
# Exec harus mengarah ke: ~/.local/bin/edbookmark-walker
```

### Window tidak floating di Hyprland

Pastikan window rules sudah ditambahkan:

```bash
grep "edbookmark" ~/.config/hypr/hyprland.conf
```

Jika belum ada, jalankan installer ulang atau tambahkan manual.

### Reset data bookmark

```bash
echo '{"version":"1.0","bookmarks":[]}' > ~/.local/share/edbookmark/bookmarks.json
```

### Reinstall jika ada masalah

```bash
bash uninstall-edbookmark.sh
bash install.sh
```

## Lisensi

[MIT License](LICENSE)

## Author

[CuedNub](https://github.com/CuedNub)

---

