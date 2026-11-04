#![windows_subsystem = "windows"]
#![allow(dead_code, unused_variables, unused_must_use)]

mod calendar;
mod capture;
mod clipboard;
mod config;
mod gps;
mod media;
mod note;
mod power;
mod prayer;
mod shelf;
mod todo;
mod window_monitor;

use calendar::CalendarGrid;
use capture::CaptureManager;
use chrono::Local;
use clipboard::ClipboardManager;
use config::Config;
use media::MediaManager;
use note::NoteManager;
use power::PowerManager;
use prayer::{calculate_times, find_city, next_prayer};
use shelf::ShelfManager;
use todo::TodoManager;

use std::sync::{Arc, Mutex};
use std::time::Instant;

#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Dwm::*,
    Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, MOD_CONTROL, MOD_SHIFT},
    Win32::UI::WindowsAndMessaging::*,
};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum ActivePanel {
    None,
    Media,
    Prayer,
    Todo,
    Note,
    Shelf,
    Clipboard,
    Power,
    Calendar,
    Capture,
}

pub struct AppState {
    pub config: Config,
    pub is_expanded: bool,
    pub current_height: i32,
    pub target_height: i32,
    pub is_mini_browser_mode: bool,
    pub active_panel: ActivePanel,
    pub media_mgr: MediaManager,
    pub shelf_mgr: ShelfManager,
    pub clip_mgr: ClipboardManager,
    pub todo_mgr: TodoManager,
    pub note_mgr: NoteManager,
    pub calendar_grid: CalendarGrid,
    pub gps_lat: f64,
    pub gps_lon: f64,
    pub gps_city: String,
    pub last_tick: Instant,
}

static mut APP_STATE: Option<Arc<Mutex<AppState>>> = None;
const HOTKEY_CAPTURE_ID: i32 = 9001;

#[cfg(target_os = "windows")]
fn main() -> Result<()> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let class_name = w!("SkadooshNotchClass");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            hInstance: instance.into(),
            lpszClassName: class_name,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            ..Default::default()
        };

        RegisterClassExW(&wc);

        let config = Config::load();
        let state = Arc::new(Mutex::new(AppState {
            config,
            is_expanded: false,
            current_height: 36,
            target_height: 36,
            is_mini_browser_mode: false,
            active_panel: ActivePanel::None,
            media_mgr: MediaManager::new(),
            shelf_mgr: ShelfManager::load(),
            clip_mgr: ClipboardManager::load(),
            todo_mgr: TodoManager::load(),
            note_mgr: NoteManager::load(),
            calendar_grid: CalendarGrid::current(),
            gps_lat: -6.2088,
            gps_lon: 106.8456,
            gps_city: "Jakarta".to_string(),
            last_tick: Instant::now(),
        }));

        APP_STATE = Some(state.clone());

        // Fetch GPS lokasi asli di background
        let state_gps = state.clone();
        gps::fetch_gps_async(move |lat, lon, city| {
            if let Ok(mut s) = state_gps.lock() {
                s.gps_lat = lat;
                s.gps_lon = lon;
                s.gps_city = city;
            }
        });

        // Hitung posisi tengah atas layar
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let notch_w = 460;
        let x = (screen_w - notch_w) / 2;

        let hwnd = CreateWindowExW(
            WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
            class_name,
            w!("Skadoosh-Notch"),
            WS_POPUP | WS_VISIBLE,
            x,
            0,
            notch_w,
            36,
            None,
            None,
            instance,
            None,
        )?;

        // Register Global HotKey Ctrl + Shift + R (0x52)
        let _ = RegisterHotKey(hwnd, HOTKEY_CAPTURE_ID, MOD_CONTROL | MOD_SHIFT, 0x52);

        // DWM dark mode & rounded corners
        let dark_mode: BOOL = TRUE;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode as *const _ as _,
            std::mem::size_of::<BOOL>() as u32,
        );

        // Alpha transparency
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 245, LWA_ALPHA);

        // Timer animasi 60 FPS (16ms)
        SetTimer(hwnd, 1, 16, None);
        // Timer update jam, sholat, auto-shrink (1 detik)
        SetTimer(hwnd, 2, 1000, None);

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).into() {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        Ok(())
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    println!("Skadoosh-Notch is a native Windows app.");
}

#[cfg(target_os = "windows")]
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            if wparam.0 == HOTKEY_CAPTURE_ID as usize {
                // Trigger Screenshot panel via Ctrl + Shift + R
                if let Some(state_arc) = &APP_STATE {
                    let mut s = state_arc.lock().unwrap();
                    s.active_panel = ActivePanel::Capture;
                    s.is_expanded = true;
                    s.target_height = 240;
                }
            }
            LRESULT(0)
        }

        WM_TIMER => {
            if wparam.0 == 1 {
                // Animasi ketinggian (Spring/Ease)
                if let Some(state_arc) = &APP_STATE {
                    let mut s = state_arc.lock().unwrap();
                    if s.current_height != s.target_height {
                        let diff = s.target_height - s.current_height;
                        let step = if diff.abs() < 2 { diff } else { diff / 4 };
                        s.current_height += step;
                        let h = s.current_height;
                        drop(s);

                        let mut rect = RECT::default();
                        let _ = GetWindowRect(hwnd, &mut rect);
                        let w = rect.right - rect.left;
                        let _ = SetWindowPos(hwnd, None, rect.left, 0, w, h, SWP_NOZORDER | SWP_NOACTIVATE);
                        let _ = InvalidateRect(hwnd, None, TRUE);
                    }
                }
            } else if wparam.0 == 2 {
                // Redraw jam & countdown
                let _ = InvalidateRect(hwnd, None, FALSE);
            }
            LRESULT(0)
        }

        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut ps);

            let mut rect = RECT::default();
            let _ = GetClientRect(hwnd, &mut rect);

            // Double buffering
            let mem_dc = CreateCompatibleDC(hdc);
            let mem_bmp = CreateCompatibleBitmap(hdc, rect.right, rect.bottom);
            let old_bmp = SelectObject(mem_dc, mem_bmp);

            // Background Notch hitam pekat rounded
            let bg_brush = CreateSolidBrush(COLORREF(0x111111));
            let rgn = CreateRoundRectRgn(0, 0, rect.right, rect.bottom, 24, 24);
            let _ = FillRgn(mem_dc, rgn, bg_brush);
            let _ = DeleteObject(rgn);
            let _ = DeleteObject(bg_brush);

            // Render Header (Jam, Sholat GPS, Icons)
            SetBkMode(mem_dc, TRANSPARENT);
            SetTextColor(mem_dc, COLORREF(0xFFFFFF));

            let now = Local::now();
            let time_str = now.format("%H:%M").to_string();

            // Hitung jadwal sholat dari GPS koordinat
            let (lat, lon) = if let Some(state_arc) = &APP_STATE {
                let s = state_arc.lock().unwrap();
                (s.gps_lat, s.gps_lon)
            } else {
                (-6.2088, 106.8456)
            };

            let p_times = calculate_times(now.date_naive(), lat, lon);
            let (next_name, _, diff_sec) = next_prayer(&p_times, &now);

            let m_until = diff_sec / 60;
            let prayer_str = format!("🕌 {} in {}m", next_name, m_until);

            // Header text
            let header = format!("{}   {}", time_str, prayer_str);
            let mut w_header: Vec<u16> = header.encode_utf16().chain(std::iter::once(0)).collect();

            let mut text_rect = RECT {
                left: 14,
                top: 8,
                right: rect.right - 14,
                bottom: 30,
            };
            DrawTextW(mem_dc, &mut w_header, &mut text_rect, DT_LEFT | DT_SINGLELINE);

            // Quick icons lengkap
            let icons = "♪  🕌  ✓  📝  📋  📂  ⏻  📅  ✂";
            let mut w_icons: Vec<u16> = icons.encode_utf16().chain(std::iter::once(0)).collect();
            let mut icon_rect = RECT {
                left: rect.right - 230,
                top: 8,
                right: rect.right - 12,
                bottom: 30,
            };
            DrawTextW(mem_dc, &mut w_icons, &mut icon_rect, DT_RIGHT | DT_SINGLELINE);

            // Jika expanded, render panel aktif
            if rect.bottom > 40 {
                let (panel_title, panel_detail) = if let Some(state_arc) = &APP_STATE {
                    let s = state_arc.lock().unwrap();
                    match s.active_panel {
                        ActivePanel::Media => (
                            "♪ Media Player & Lyrics (LRCLIB)",
                            format!("Track: {}\nArtis: {}\n\nLirik:\n{}", s.media_mgr.state.lock().unwrap().title, s.media_mgr.state.lock().unwrap().artist, s.media_mgr.state.lock().unwrap().lyrics),
                        ),
                        ActivePanel::Prayer => (
                            "🕌 Jadwal Sholat (Kemenag RI - GPS)",
                            format!(
                                "Lokasi: {}\nSubuh:   {}\nDzuhur:  {}\nAshar:   {}\nMaghrib: {}\nIsya:    {}\n\nAuto-pause musik saat adzan: Aktif",
                                s.gps_city,
                                p_times.fajr.format("%H:%M"),
                                p_times.dhuhr.format("%H:%M"),
                                p_times.asr.format("%H:%M"),
                                p_times.maghrib.format("%H:%M"),
                                p_times.isha.format("%H:%M"),
                            ),
                        ),
                        ActivePanel::Todo => (
                            "✓ To-Do List (dengan Tanggal)",
                            if s.todo_mgr.items.is_empty() {
                                "Belum ada tugas. Klik kanan untuk tambah tugas.".to_string()
                            } else {
                                s.todo_mgr.items.iter().enumerate().map(|(i, t)| {
                                    let status = if t.done { "[✓]" } else { "[ ]" };
                                    let dl = t.deadline.as_deref().unwrap_or("Tanpa deadline");
                                    format!("{} {} {} ({})", i + 1, status, t.title, dl)
                                }).collect::<Vec<_>>().join("\n")
                            },
                        ),
                        ActivePanel::Note => (
                            "📝 Quick Notes (Auto-save)",
                            if s.note_mgr.items.is_empty() {
                                "Scratchpad kosong. Ketik catatan cepat di sini.".to_string()
                            } else {
                                s.note_mgr.items[0].content.clone()
                            },
                        ),
                        ActivePanel::Shelf => (
                            "📂 File Shelf (Unlimited)",
                            if s.shelf_mgr.items.is_empty() {
                                "Tarik / Drag file ke sini untuk disimpan sementara.".to_string()
                            } else {
                                s.shelf_mgr.items.join("\n")
                            },
                        ),
                        ActivePanel::Clipboard => (
                            "📋 Clipboard History (Unlimited)",
                            if s.clip_mgr.history.is_empty() {
                                "Riwayat copy teks akan muncul otomatis di sini.".to_string()
                            } else {
                                s.clip_mgr.history.iter().take(6).enumerate().map(|(i, t)| format!("{}. {}", i + 1, t)).collect::<Vec<_>>().join("\n")
                            },
                        ),
                        ActivePanel::Power => (
                            "⏻ Power & Layar Timeout",
                            "1. Lock PC: Win+L\n2. Layar Mati: Otomatis via SC_MONITORPOWER\n3. Sleep PC: Standby\n4. Timeout Layar: 10 menit (bisa diatur)".to_string(),
                        ),
                        ActivePanel::Calendar => (
                            "📅 Kalender Mini",
                            format!(
                                "Bulan: {} / {}\nHari ini: Tanggal {}\nTotal hari: {}",
                                s.calendar_grid.month, s.calendar_grid.year, s.calendar_grid.today_day, s.calendar_grid.days_in_month
                            ),
                        ),
                        ActivePanel::Capture => (
                            "✂ Screenshot & Record (Ctrl+Shift+R)",
                            "Pilihan Tangkapan Layar:\n• Klik 1: Snip Sebagian Layar\n• Klik 2: Full Screen Capture\n• Klik 3: Screen Record (Game Bar)".to_string(),
                        ),
                        ActivePanel::None => ("Dashboard", "Klik ikon di kanan atas untuk membuka panel.".to_string()),
                    }
                } else {
                    ("Panel", String::new())
                };

                // Judul panel warna aksen hijau
                SetTextColor(mem_dc, COLORREF(0x4CAF50));
                let mut w_title: Vec<u16> = panel_title.encode_utf16().chain(std::iter::once(0)).collect();
                let mut title_rect = RECT {
                    left: 18,
                    top: 44,
                    right: rect.right - 18,
                    bottom: 66,
                };
                DrawTextW(mem_dc, &mut w_title, &mut title_rect, DT_LEFT | DT_SINGLELINE);

                // Konten isi panel
                SetTextColor(mem_dc, COLORREF(0xCCCCCC));
                let mut w_detail: Vec<u16> = panel_detail.encode_utf16().chain(std::iter::once(0)).collect();
                let mut detail_rect = RECT {
                    left: 18,
                    top: 70,
                    right: rect.right - 18,
                    bottom: rect.bottom - 10,
                };
                DrawTextW(mem_dc, &mut w_detail, &mut detail_rect, DT_LEFT | DT_WORDBREAK);
            }

            // Copy back-buffer ke layar
            let _ = BitBlt(hdc, 0, 0, rect.right, rect.bottom, mem_dc, 0, 0, SRCCOPY);

            // Cleanup GDI
            let _ = SelectObject(mem_dc, old_bmp);
            let _ = DeleteObject(mem_bmp);
            let _ = DeleteDC(mem_dc);

            EndPaint(hwnd, &ps);
            LRESULT(0)
        }

        WM_LBUTTONDOWN => {
            // Toggle Expand / Collapse
            if let Some(state_arc) = &APP_STATE {
                let mut s = state_arc.lock().unwrap();
                s.is_expanded = !s.is_expanded;
                s.target_height = if s.is_expanded { 240 } else { 36 };
                if s.is_expanded && s.active_panel == ActivePanel::None {
                    s.active_panel = ActivePanel::Prayer;
                }
            }
            LRESULT(0)
        }

        WM_RBUTTONDOWN => {
            // Rotasi switch panel:
            // Media -> Prayer -> Todo -> Note -> Clipboard -> Shelf -> Power -> Calendar -> Capture
            if let Some(state_arc) = &APP_STATE {
                let mut s = state_arc.lock().unwrap();
                s.active_panel = match s.active_panel {
                    ActivePanel::None => ActivePanel::Prayer,
                    ActivePanel::Prayer => ActivePanel::Media,
                    ActivePanel::Media => ActivePanel::Todo,
                    ActivePanel::Todo => ActivePanel::Note,
                    ActivePanel::Note => ActivePanel::Clipboard,
                    ActivePanel::Clipboard => ActivePanel::Shelf,
                    ActivePanel::Shelf => ActivePanel::Power,
                    ActivePanel::Power => ActivePanel::Calendar,
                    ActivePanel::Calendar => ActivePanel::Capture,
                    ActivePanel::Capture => ActivePanel::Prayer,
                };
                s.is_expanded = true;
                s.target_height = 240;
            }
            LRESULT(0)
        }

        WM_DESTROY => {
            PostQuitMessage(0);
            LRESULT(0)
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
