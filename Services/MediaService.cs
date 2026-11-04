using System;
using System.IO;
using System.Net.Http;
using System.Text.Json;
using System.Threading.Tasks;
using System.Windows.Media.Imaging;
using Windows.Media.Control;
using Windows.Storage.Streams;

namespace SkadooshNotch.Services;

public class MediaService
{
    private GlobalSystemMediaTransportControlsSessionManager? _manager;
    private GlobalSystemMediaTransportControlsSession? _currentSession;
    private static readonly HttpClient _http = new() { Timeout = TimeSpan.FromSeconds(4) };

    public string Title { get; private set; } = "Tidak ada media";
    public string Artist { get; private set; } = "";
    public bool IsPlaying { get; private set; }
    public string CurrentLyrics { get; private set; } = "";
    public BitmapImage? Thumbnail { get; private set; }

    public event Action? OnMediaChanged;

    public async Task InitializeAsync()
    {
        try
        {
            _manager = await GlobalSystemMediaTransportControlsSessionManager.RequestAsync();
            if (_manager != null)
            {
                _manager.CurrentSessionChanged += async (s, e) => await UpdateSessionAsync();
                await UpdateSessionAsync();
            }
        }
        catch { }
    }

    public async Task UpdateSessionAsync()
    {
        if (_manager == null) return;

        try
        {
            _currentSession = _manager.GetCurrentSession();
            if (_currentSession == null)
            {
                Title = "Tidak ada media";
                Artist = "";
                IsPlaying = false;
                CurrentLyrics = "";
                Thumbnail = null;
                OnMediaChanged?.Invoke();
                return;
            }

            var playback = _currentSession.GetPlaybackInfo();
            IsPlaying = playback?.PlaybackStatus == GlobalSystemMediaTransportControlsSessionPlaybackStatus.Playing;

            var props = await _currentSession.TryGetMediaPropertiesAsync();
            if (props != null)
            {
                string newTitle = props.Title ?? "Unknown Title";
                string newArtist = props.Artist ?? "Unknown Artist";

                if (newTitle != Title || newArtist != Artist)
                {
                    Title = newTitle;
                    Artist = newArtist;
                    _ = FetchLyricsAsync(Title, Artist);
                }

                if (props.Thumbnail != null)
                {
                    Thumbnail = await LoadThumbnailAsync(props.Thumbnail);
                }
            }

            OnMediaChanged?.Invoke();
        }
        catch { }
    }

    public async Task TogglePlayPauseAsync()
    {
        if (_currentSession == null) return;
        try
        {
            await _currentSession.TryTogglePlayPauseAsync();
            await UpdateSessionAsync();
        }
        catch { }
    }

    public async Task NextTrackAsync()
    {
        if (_currentSession == null) return;
        try
        {
            await _currentSession.TrySkipNextAsync();
            await UpdateSessionAsync();
        }
        catch { }
    }

    public async Task PreviousTrackAsync()
    {
        if (_currentSession == null) return;
        try
        {
            await _currentSession.TrySkipPreviousAsync();
            await UpdateSessionAsync();
        }
        catch { }
    }

    private async Task FetchLyricsAsync(string title, string artist)
    {
        CurrentLyrics = "Mencari lirik...";
        OnMediaChanged?.Invoke();

        try
        {
            var url = $"https://lrclib.net/api/get?track_name={Uri.EscapeDataString(title)}&artist_name={Uri.EscapeDataString(artist)}";
            var json = await _http.GetStringAsync(url);
            using var doc = JsonDocument.Parse(json);

            if (doc.RootElement.TryGetProperty("plainLyrics", out var plain) && plain.GetString() is string lyrics && !string.IsNullOrWhiteSpace(lyrics))
            {
                CurrentLyrics = lyrics;
            }
            else if (doc.RootElement.TryGetProperty("syncedLyrics", out var synced) && synced.GetString() is string sLyrics)
            {
                CurrentLyrics = sLyrics;
            }
            else
            {
                CurrentLyrics = "Lirik tidak ditemukan.";
            }
        }
        catch
        {
            CurrentLyrics = "Lirik offline / tidak ditemukan.";
        }

        OnMediaChanged?.Invoke();
    }

    private static async Task<BitmapImage?> LoadThumbnailAsync(IRandomAccessStreamReference streamRef)
    {
        try
        {
            using var stream = await streamRef.OpenReadAsync();
            var bytes = new byte[stream.Size];
            using (var reader = new DataReader(stream))
            {
                await reader.LoadAsync((uint)stream.Size);
                reader.ReadBytes(bytes);
            }
            using var ms = new MemoryStream(bytes);
            var bmp = new BitmapImage();
            bmp.BeginInit();
            bmp.CacheOption = BitmapCacheOption.OnLoad;
            bmp.StreamSource = ms;
            bmp.EndInit();
            bmp.Freeze();
            return bmp;
        }
        catch { return null; }
    }
}
