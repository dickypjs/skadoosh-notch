#![windows_subsystem = "windows"]
#![allow(dead_code, unused_variables, unused_must_use)]

mod clipboard;
mod config;
mod media;
mod power;
mod prayer;
mod shelf;

use chrono::Local;
use config::Config;
use media::MediaManager;
use prayer::{calculate_times, find_city, next_prayer};
use shelf::ShelfManager;
use clipboard::ClipboardManager;


use std::sync::{Arc, Mutex};
use std::time::Instant;

#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::Graphics::Gdi::*,
    Win32::Graphics::Dwm::*,
    Win32::System::LibraryLoader::GetModuleHandleW,
    Win32::UI::WindowsAndMessaging::*,
};

#[derive(PartialEq, Clone, Copy)]
pub enum ActivePanel {
    None,
    Media,
    Prayer,
    Shelf,
    Clipboard,
    Power,
}

pub struct AppState {
    pub config: Config,
    pub is_expanded: bool,
    pub current_height: i32,
    pub target_height: i32,
    pub active_panel: ActivePanel,
    pub media_mgr: MediaManager,
    pub shelf_mgr: ShelfManager,
    pub clip_mgr: ClipboardManager,
    pub last_tick: Instant,
}

static mut APP_STATE: Option<Arc<Mutex<AppState>>> = None;

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
            active_panel: ActivePanel::None,
            media_mgr: MediaManager::new(),
            shelf_mgr: ShelfManager::load(),
            clip_mgr: ClipboardManager::load(),
            last_tick: Instant::now(),
        }));

        APP_STATE = Some(state.clone());

        // Hitung posisi tengah atas layar
        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let notch_w = 420;
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

        // Set DWM rounded corners & dark mode
        let dark_mode: BOOL = TRUE;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_USE_IMMERSIVE_DARK_MODE,
            &dark_mode as *const _ as _,
            std::mem::size_of::<BOOL>() as u32,
        );

        // Alpha transparency
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), 240, LWA_ALPHA);

        // Timer animasi 60 FPS (16ms)
        SetTimer(hwnd, 1, 16, None);
        // Timer update jam / sholat (1 detik)
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
                // Check waktu sholat & polling
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

            // Render Header (Jam, Sholat, Icons)
            SetBkMode(mem_dc, TRANSPARENT);
            SetTextColor(mem_dc, COLORREF(0xFFFFFF));

            let now = Local::now();
            let time_str = now.format("%H:%M").to_string();

            // Hitung jadwal sholat kota aktif
            let city_name = if let Some(state_arc) = &APP_STATE {
                state_arc.lock().unwrap().config.city.clone()
            } else {
                "Jakarta".to_string()
            };
            let coord = find_city(&city_name);
            let p_times = calculate_times(now.date_naive(), coord.lat, coord.lon);
            let (next_name, _, diff_sec) = next_prayer(&p_times, &now);

            let m_until = diff_sec / 60;
            let prayer_str = format!("{} in {}m", next_name, m_until);

            // Header text
            let header = format!("{}   |   {}", time_str, prayer_str);
            let mut w_header: Vec<u16> = header.encode_utf16().chain(std::iter::once(0)).collect();

            let mut text_rect = RECT {
                left: 16,
                top: 8,
                right: rect.right - 16,
                bottom: 30,
            };
            DrawTextW(mem_dc, &mut w_header, &mut text_rect, DT_LEFT | DT_SINGLELINE);

            // Quick icons di sebelah kanan
            let icons = "♪   🕌   📂   📋   ⏻";
            let mut w_icons: Vec<u16> = icons.encode_utf16().chain(std::iter::once(0)).collect();
            let mut icon_rect = RECT {
                left: rect.right - 180,
                top: 8,
                right: rect.right - 16,
                bottom: 30,
            };
            DrawTextW(mem_dc, &mut w_icons, &mut icon_rect, DT_RIGHT | DT_SINGLELINE);

            // Jika expanded, render panel konten
            if rect.bottom > 40 {
                let panel_title = if let Some(state_arc) = &APP_STATE {
                    match state_arc.lock().unwrap().active_panel {
                        ActivePanel::Media => "Media Player & Lyrics (LRCLIB)",
                        ActivePanel::Prayer => "Jadwal Sholat (Kemenag RI)",
                        ActivePanel::Shelf => "File Shelf (Drop / Unlimited)",
                        ActivePanel::Clipboard => "Clipboard History",
                        ActivePanel::Power => "Power & Screen Off",
                        ActivePanel::None => "Quick Dashboard",
                    }
                } else {
                    "Panel"
                };

                SetTextColor(mem_dc, COLORREF(0x4CAF50));
                let mut w_panel: Vec<u16> = panel_title.encode_utf16().chain(std::iter::once(0)).collect();
                let mut content_rect = RECT {
                    left: 20,
                    top: 48,
                    right: rect.right - 20,
                    bottom: rect.bottom - 10,
                };
                DrawTextW(mem_dc, &mut w_panel, &mut content_rect, DT_LEFT | DT_WORDBREAK);
            }

            // Copy back-buffer to screen
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
            // Siklus switch panel: Media -> Prayer -> Shelf -> Clipboard -> Power
            if let Some(state_arc) = &APP_STATE {
                let mut s = state_arc.lock().unwrap();
                s.active_panel = match s.active_panel {
                    ActivePanel::None => ActivePanel::Prayer,
                    ActivePanel::Prayer => ActivePanel::Media,
                    ActivePanel::Media => ActivePanel::Shelf,
                    ActivePanel::Shelf => ActivePanel::Clipboard,
                    ActivePanel::Clipboard => ActivePanel::Power,
                    ActivePanel::Power => ActivePanel::Prayer,
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
