using System;
using System.IO;
using System.Net.Http;
using System.Text.Json;
using System.Threading.Tasks;
using System.Windows.Media;
using Windows.Devices.Geolocation;
using SkadooshNotch.Models;

namespace SkadooshNotch.Services;

public class PrayerService
{
    private readonly MediaPlayer _player = new();
    private static readonly HttpClient _http = new() { Timeout = TimeSpan.FromSeconds(5) };

    public double Latitude { get; private set; } = -6.2088; // Default: Jakarta
    public double Longitude { get; private set; } = 106.8456;
    public string SoundPath { get; set; } = "";
    public PrayerTime CurrentTimes { get; private set; } = new();

    public event Action<string>? OnPrayerArrived;

    public async Task InitLocationAsync()
    {
        try
        {
            var access = await Geolocator.RequestAccessAsync();
            if (access == GeolocationAccessStatus.Allowed)
            {
                var locator = new Geolocator { DesiredAccuracy = PositionAccuracy.Default };
                var pos = await locator.GetGeopositionAsync();
                Latitude = pos.Coordinate.Point.Position.Latitude;
                Longitude = pos.Coordinate.Point.Position.Longitude;
            }
            else
            {
                await FallbackIpLocationAsync();
            }
        }
        catch
        {
            await FallbackIpLocationAsync();
        }

        CalculateTimes();
    }

    private async Task FallbackIpLocationAsync()
    {
        try
        {
            var res = await _http.GetStringAsync("https://ipapi.co/json/");
            using var doc = JsonDocument.Parse(res);
            if (doc.RootElement.TryGetProperty("latitude", out var lat))
                Latitude = lat.GetDouble();
            if (doc.RootElement.TryGetProperty("longitude", out var lon))
                Longitude = lon.GetDouble();
        }
        catch { /* Gunakan default Jakarta */ }
    }

    public void CalculateTimes()
    {
        // Standar Kemenag RI: Subuh 20°, Isya 18°, Dzuhur transit, Ashar bayangan 1x
        var today = DateTime.Today;
        CurrentTimes = CalculateForDate(today, Latitude, Longitude);
    }

    public void CheckAlarm(DateTime now)
    {
        var times = new (string Name, DateTime Time)[]
        {
            ("Subuh", CurrentTimes.Fajr),
            ("Dzuhur", CurrentTimes.Dhuhr),
            ("Ashar", CurrentTimes.Asr),
            ("Maghrib", CurrentTimes.Maghrib),
            ("Isya", CurrentTimes.Isha)
        };

        foreach (var (name, time) in times)
        {
            if (Math.Abs((now - time).TotalSeconds) < 1)
            {
                PlaySound();
                OnPrayerArrived?.Invoke(name);
                break;
            }
        }
    }

    public void PlaySound()
    {
        try
        {
            if (!string.IsNullOrEmpty(SoundPath) && File.Exists(SoundPath))
            {
                _player.Open(new Uri(SoundPath));
                _player.Play();
            }
            else
            {
                System.Media.SystemSounds.Exclamation.Play();
            }
        }
        catch { }
    }

    private static PrayerTime CalculateForDate(DateTime date, double lat, double lon)
    {
        // Rumus sederhana sudut matahari (Julian day approximation)
        double d = (date - new DateTime(2000, 1, 1, 12, 0, 0, DateTimeKind.Utc)).TotalDays;
        double q = 280.459 + 0.98564736 * d;
        double g = 357.529 + 0.98560028 * d;
        double l = q + 1.915 * Math.Sin(g * Math.PI / 180) + 0.020 * Math.Sin(2 * g * Math.PI / 180);
        double e = 23.439 - 0.00000036 * d;
        double ra = Math.Atan2(Math.Cos(e * Math.PI / 180) * Math.Sin(l * Math.PI / 180), Math.Cos(l * Math.PI / 180)) * 180 / Math.PI;
        double dAngle = q - ra;
        double eqTime = 4 * dAngle; // minutes
        double decl = Math.Asin(Math.Sin(e * Math.PI / 180) * Math.Sin(l * Math.PI / 180)) * 180 / Math.PI;

        double timezone = Math.Round(lon / 15.0);
        double noon = 12 + (timezone * 15 - lon) / 15.0 - eqTime / 60.0;

        double CalcHourAngle(double angle)
        {
            double rad = Math.PI / 180.0;
            double val = (-Math.Sin(angle * rad) - Math.Sin(lat * rad) * Math.Sin(decl * rad)) /
                         (Math.Cos(lat * rad) * Math.Cos(decl * rad));
            if (val > 1) val = 1; if (val < -1) val = -1;
            return Math.Acos(val) * 180.0 / Math.PI / 15.0;
        }

        // Subuh: 20 derajat dibawah ufuk
        double fajrHa = CalcHourAngle(20.0);
        // Isya: 18 derajat
        double ishaHa = CalcHourAngle(18.0);
        // Maghrib: 0.833 derajat (refraksi)
        double maghribHa = CalcHourAngle(0.833);
        // Ashar: bayangan + 1
        double asrAngle = -Math.Atan(1.0 + Math.Tan(Math.Abs(lat - decl) * Math.PI / 180.0)) * 180.0 / Math.PI;
        double asrHa = CalcHourAngle(-asrAngle);

        DateTime ToDate(double h)
        {
            int hour = (int)Math.Floor(h);
            int min = (int)Math.Floor((h - hour) * 60);
            int sec = (int)Math.Round(((h - hour) * 60 - min) * 60);
            return date.Date.Add(new TimeSpan(hour, min, sec));
        }

        return new PrayerTime
        {
            Fajr    = ToDate(noon - fajrHa),
            Dhuhr   = ToDate(noon + (2.0 / 60.0)), // Ihtiyat +2 menit
            Asr     = ToDate(noon + asrHa),
            Maghrib = ToDate(noon + maghribHa + (2.0 / 60.0)),
            Isha    = ToDate(noon + ishaHa)
        };
    }
}
