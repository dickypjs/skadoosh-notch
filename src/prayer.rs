use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, NaiveTime, Timelike};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct PrayerTimes {
    pub fajr: NaiveDateTime,
    pub dhuhr: NaiveDateTime,
    pub asr: NaiveDateTime,
    pub maghrib: NaiveDateTime,
    pub isha: NaiveDateTime,
}

pub struct CityCoord {
    pub name: &'static str,
    pub province: &'static str,
    pub lat: f64,
    pub lon: f64,
}

pub const CITIES: &[CityCoord] = &[
    CityCoord { name: "Banda Aceh", province: "Aceh", lat: 5.5483, lon: 95.3238 },
    CityCoord { name: "Lhokseumawe", province: "Aceh", lat: 5.1801, lon: 97.1507 },
    CityCoord { name: "Medan", province: "Sumatera Utara", lat: 3.5952, lon: 98.6722 },
    CityCoord { name: "Pematangsiantar", province: "Sumatera Utara", lat: 2.9610, lon: 99.0682 },
    CityCoord { name: "Padang", province: "Sumatera Barat", lat: -0.9471, lon: 100.4172 },
    CityCoord { name: "Bukittinggi", province: "Sumatera Barat", lat: -0.3055, lon: 100.3692 },
    CityCoord { name: "Pekanbaru", province: "Riau", lat: 0.5071, lon: 101.4478 },
    CityCoord { name: "Batam", province: "Kepulauan Riau", lat: 1.1293, lon: 104.0534 },
    CityCoord { name: "Palembang", province: "Sumatera Selatan", lat: -2.9761, lon: 104.7754 },
    CityCoord { name: "Bandar Lampung", province: "Lampung", lat: -5.4500, lon: 105.2667 },
    CityCoord { name: "Jakarta", province: "DKI Jakarta", lat: -6.2088, lon: 106.8456 },
    CityCoord { name: "Bandung", province: "Jawa Barat", lat: -6.9175, lon: 107.6191 },
    CityCoord { name: "Bekasi", province: "Jawa Barat", lat: -6.2383, lon: 106.9756 },
    CityCoord { name: "Bogor", province: "Jawa Barat", lat: -6.5971, lon: 106.8060 },
    CityCoord { name: "Depok", province: "Jawa Barat", lat: -6.4025, lon: 106.7942 },
    CityCoord { name: "Semarang", province: "Jawa Tengah", lat: -6.9667, lon: 110.4167 },
    CityCoord { name: "Surakarta", province: "Jawa Tengah", lat: -7.5755, lon: 110.8243 },
    CityCoord { name: "Yogyakarta", province: "DI Yogyakarta", lat: -7.7956, lon: 110.3695 },
    CityCoord { name: "Surabaya", province: "Jawa Timur", lat: -7.2575, lon: 112.7521 },
    CityCoord { name: "Malang", province: "Jawa Timur", lat: -7.9666, lon: 112.6326 },
    CityCoord { name: "Denpasar", province: "Bali", lat: -8.6705, lon: 115.2126 },
    CityCoord { name: "Mataram", province: "Nusa Tenggara Barat", lat: -8.5833, lon: 116.1167 },
    CityCoord { name: "Kupang", province: "Nusa Tenggara Timur", lat: -10.1772, lon: 123.6070 },
    CityCoord { name: "Pontianak", province: "Kalimantan Barat", lat: -0.0263, lon: 109.3425 },
    CityCoord { name: "Banjarmasin", province: "Kalimantan Selatan", lat: -3.3194, lon: 114.5908 },
    CityCoord { name: "Samarinda", province: "Kalimantan Timur", lat: -0.5022, lon: 117.1536 },
    CityCoord { name: "Makassar", province: "Sulawesi Selatan", lat: -5.1477, lon: 119.4327 },
    CityCoord { name: "Manado", province: "Sulawesi Utara", lat: 1.4748, lon: 124.8428 },
    CityCoord { name: "Ambon", province: "Maluku", lat: -3.6554, lon: 128.1908 },
    CityCoord { name: "Jayapura", province: "Papua", lat: -2.5916, lon: 140.6690 },
];

pub fn find_city(name: &str) -> &'static CityCoord {
    CITIES.iter().find(|c| c.name.eq_ignore_ascii_case(name)).unwrap_or(&CITIES[10]) // default Jakarta
}

// Standar Kemenag RI:
// Subuh: 20 derajat, Isya: 18 derajat, Dzuhur: matahari transit + ihtiyat, Ashar: bayangan + 1
pub fn calculate_times(date: NaiveDate, lat: f64, lon: f64) -> PrayerTimes {
    let day_of_year = date.ordinal() as f64;
    let b = 2.0 * PI * (day_of_year - 81.0) / 365.0;
    let eq_time = 9.87 * (2.0 * b).sin() - 7.53 * b.cos() - 1.5 * b.sin(); // menit
    let decl = 23.45 * (b.sin()); // derajat

    let timezone = (lon / 15.0).round();
    let noon = 12.0 + (timezone * 15.0 - lon) / 15.0 - eq_time / 60.0;

    let hour_angle = |angle: f64| -> f64 {
        let rad = PI / 180.0;
        let val = (-angle.to_radians().sin() - (lat * rad).sin() * (decl * rad).sin())
            / ((lat * rad).cos() * (decl * rad).cos());
        let clamped = val.clamp(-1.0, 1.0);
        clamped.acos() * 180.0 / PI / 15.0
    };

    let fajr_ha = hour_angle(20.0);
    let isha_ha = hour_angle(18.0);
    let maghrib_ha = hour_angle(0.833);

    // Ashar: bayangan + 1
    let asr_angle = -((1.0 + (lat - decl).abs().to_radians().tan()).atan() * 180.0 / PI);
    let asr_ha = hour_angle(-asr_angle);

    let to_dt = |hours: f64| -> NaiveDateTime {
        let h = hours.floor() as u32;
        let rem_m = (hours - hours.floor()) * 60.0;
        let m = rem_m.floor() as u32;
        let s = ((rem_m - rem_m.floor()) * 60.0).round() as u32;
        let t = NaiveTime::from_hms_opt(h.min(23), m.min(59), s.min(59)).unwrap_or_default();
        date.and_time(t)
    };

    PrayerTimes {
        fajr: to_dt(noon - fajr_ha),
        dhuhr: to_dt(noon + 2.0 / 60.0), // Ihtiyat 2 menit
        asr: to_dt(noon + asr_ha),
        maghrib: to_dt(noon + maghrib_ha + 2.0 / 60.0),
        isha: to_dt(noon + isha_ha),
    }
}

pub fn next_prayer(times: &PrayerTimes, now: &DateTime<Local>) -> (&'static str, NaiveDateTime, i64) {
    let now_naive = now.naive_local();
    let prayers = [
        ("Subuh", times.fajr),
        ("Dzuhur", times.dhuhr),
        ("Ashar", times.asr),
        ("Maghrib", times.maghrib),
        ("Isya", times.isha),
    ];

    for &(name, time) in &prayers {
        if time > now_naive {
            let diff_sec = (time - now_naive).num_seconds();
            return (name, time, diff_sec);
        }
    }

    // Lewat Isya -> Subuh besok (+24 jam)
    let next_fajr = times.fajr + chrono::Duration::days(1);
    let diff_sec = (next_fajr - now_naive).num_seconds();
    ("Subuh", next_fajr, diff_sec)
}
