use serde::Deserialize;
use std::thread;

#[derive(Debug, Clone, Copy, Default)]
pub struct GeoCoord {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Deserialize)]
struct IpApiResp {
    latitude: Option<f64>,
    longitude: Option<f64>,
    city: Option<String>,
}

pub fn fetch_gps_async<F>(callback: F)
where
    F: FnOnce(f64, f64, String) + Send + 'static,
{
    thread::spawn(move || {
        // Coba endpoint IP Geolocation akurat & gratis
        if let Ok(resp) = ureq::get("https://ipapi.co/json/").timeout(std::time::Duration::from_secs(4)).call() {
            if let Ok(info) = resp.into_json::<IpApiResp>() {
                if let (Some(lat), Some(lon)) = (info.latitude, info.longitude) {
                    let city = info.city.unwrap_or_else(|| "Lokasi GPS".to_string());
                    callback(lat, lon, city);
                    return;
                }
            }
        }
        // Fallback default Jakarta
        callback(-6.2088, 106.8456, "Jakarta".to_string());
    });
}
