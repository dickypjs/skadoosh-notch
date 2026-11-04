use serde::Deserialize;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Default)]
pub struct MediaState {
    pub title: String,
    pub artist: String,
    pub is_playing: bool,
    pub lyrics: String,
}

#[derive(Deserialize)]
struct LrcResponse {
    #[serde(rename = "plainLyrics")]
    plain_lyrics: Option<String>,
    #[serde(rename = "syncedLyrics")]
    synced_lyrics: Option<String>,
}

pub struct MediaManager {
    pub state: Arc<Mutex<MediaState>>,
}

impl MediaManager {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MediaState::default())),
        }
    }

    pub fn fetch_lyrics_async(&self, title: String, artist: String) {
        let state_clone = self.state.clone();
        thread::spawn(move || {
            {
                let mut s = state_clone.lock().unwrap();
                s.lyrics = "Mencari lirik...".to_string();
            }

            let url = format!(
                "https://lrclib.net/api/get?track_name={}&artist_name={}",
                urlencoding(&title),
                urlencoding(&artist)
            );

            let lyrics = match ureq::get(&url).timeout(std::time::Duration::from_secs(4)).call() {
                Ok(resp) => {
                    if let Ok(lrc) = resp.into_json::<LrcResponse>() {
                        lrc.plain_lyrics.or(lrc.synced_lyrics).unwrap_or_else(|| "Lirik tidak ditemukan.".to_string())
                    } else {
                        "Format lirik tidak cocok.".to_string()
                    }
                }
                Err(_) => "Lirik offline / tidak ditemukan.".to_string(),
            };

            let mut s = state_clone.lock().unwrap();
            s.lyrics = lyrics;
        });
    }

    pub fn update_track(&self, title: String, artist: String, is_playing: bool) {
        let mut s = self.state.lock().unwrap();
        let changed = s.title != title || s.artist != artist;
        s.title = title.clone();
        s.artist = artist.clone();
        s.is_playing = is_playing;

        if changed && !title.is_empty() {
            drop(s);
            self.fetch_lyrics_async(title, artist);
        }
    }
}

fn urlencoding(input: &str) -> String {
    let mut encoded = String::new();
    for b in input.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}
