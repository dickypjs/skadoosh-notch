using Microsoft.Win32;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using System.Windows.Media;
using System.Windows.Media.Animation;
using System.Windows.Threading;
using SkadooshNotch.Models;
using SkadooshNotch.Services;

namespace SkadooshNotch.Views;

public partial class MainWindow : Window
{
    // Services
    private readonly PrayerService _prayer = new();
    private readonly MediaService _media = new();
    private readonly ClipboardService _clipboard = new();
    private readonly TodoService _todo = new();
    private readonly NoteService _note = new();
    private readonly ShelfService _shelf = new();
    private readonly WindowMonitorService _winMonitor = new();
    private readonly HotKeyService _hotKey = new();

    // Timers
    private readonly DispatcherTimer _clockTimer = new() { Interval = TimeSpan.FromSeconds(1) };
    private readonly DispatcherTimer _clipTimer = new() { Interval = TimeSpan.FromMilliseconds(800) };
    private readonly DispatcherTimer _monitorTimer = new() { Interval = TimeSpan.FromSeconds(2) };
    private readonly DispatcherTimer _prayerCheckTimer = new() { Interval = TimeSpan.FromSeconds(1) };

    private bool _isExpanded;
    private string? _activePanelName;
    private NoteItem? _activeNote;

    public MainWindow()
    {
        InitializeComponent();
        Loaded += OnLoaded;
        Closing += OnClosing;
    }

    private async void OnLoaded(object sender, RoutedEventArgs e)
    {
        PositionNotch();
        InitTimers();
        BindCollections();

        // HotKey
        _hotKey.Register(this);
        _hotKey.OnHotKeyPressed += () => Dispatcher.Invoke(ShowCapturePanel);

        // Prayer init
        _prayer.OnPrayerArrived += name => Dispatcher.Invoke(() =>
        {
            new System.Windows.Controls.ToolTip { Content = $"Waktu {name} telah tiba!" }.IsOpen = true;
        });
        await _prayer.InitLocationAsync();
        RefreshPrayerList();

        // Media
        _media.OnMediaChanged += () => Dispatcher.Invoke(RefreshMediaUI);
        await _media.InitializeAsync();

        // Window monitor
        _winMonitor.OnStateChanged += (browser, fullscreen) => Dispatcher.Invoke(() =>
        {
            if (fullscreen)
                Visibility = Visibility.Hidden;
            else
            {
                Visibility = Visibility.Visible;
                if (browser && !_isExpanded)
                    BeginStoryboard((Storyboard)Resources["MiniAnim"]);
                else if (!browser && !_isExpanded)
                    CollapseToNormal();
            }
        });
    }

    private void OnClosing(object? sender, System.ComponentModel.CancelEventArgs e)
    {
        _hotKey.Dispose();
        _note.Save();
        _todo.Save();
    }

    private void PositionNotch()
    {
        var screen = System.Windows.Forms.Screen.PrimaryScreen?.Bounds;
        double screenW = screen?.Width ?? 1920;
        double dpi = PresentationSource.FromVisual(this)?.CompositionTarget?.TransformToDevice.M11 ?? 1.0;
        Left = (screenW / dpi - 400) / 2;
        Top = 0;
        Width = 400;
    }

    private void InitTimers()
    {
        _clockTimer.Tick += (_, _) => TickClock();
        _clockTimer.Start();

        _clipTimer.Tick += (_, _) => _clipboard.PollClipboard();
        _clipTimer.Start();

        _monitorTimer.Tick += (_, _) => _winMonitor.CheckActiveWindow();
        _monitorTimer.Start();

        _prayerCheckTimer.Tick += (_, _) => _prayer.CheckAlarm(DateTime.Now);
        _prayerCheckTimer.Start();
    }

    private void BindCollections()
    {
        TodoList.ItemsSource = _todo.Items;
        ClipList.ItemsSource = _clipboard.History;
        ShelfList.ItemsSource = _shelf.Items;
        _clipboard.OnClipboardChanged += () => Dispatcher.Invoke(() => { });
    }

    // ─── Clock ───────────────────────────────────────────────
    private void TickClock()
    {
        TbClock.Text = DateTime.Now.ToString("HH:mm");

        var (name, time) = _prayer.CurrentTimes.Next();
        var diff = time - DateTime.Now;
        TbPrayer.Text = diff.TotalMinutes < 1
            ? $"⏰ {name}"
            : $"{name} {(int)diff.TotalHours}:{diff.Minutes:D2}";
    }

    // ─── Media UI ────────────────────────────────────────────
    private void RefreshMediaUI()
    {
        TbMediaPill.Text = _media.IsPlaying ? $"♪ {_media.Title}" : "";
        TbMediaTitle.Text = _media.Title;
        TbMediaArtist.Text = _media.Artist;
        BtnPlayPause.Content = _media.IsPlaying ? "⏸" : "▶";
        TbLyrics.Text = _media.CurrentLyrics;

        if (_media.Thumbnail != null)
            MediaThumbBrush.ImageSource = _media.Thumbnail;
    }

    private async void BtnPlayPause_Click(object s, RoutedEventArgs e) => await _media.TogglePlayPauseAsync();
    private async void BtnPrev_Click(object s, RoutedEventArgs e) => await _media.PreviousTrackAsync();
    private async void BtnNext_Click(object s, RoutedEventArgs e) => await _media.NextTrackAsync();

    // ─── Prayer ──────────────────────────────────────────────
    private void RefreshPrayerList()
    {
        var t = _prayer.CurrentTimes;
        PrayerList.ItemsSource = new[]
        {
            new { Name = "Subuh",   TimeStr = t.Fajr.ToString("HH:mm") },
            new { Name = "Dzuhur",  TimeStr = t.Dhuhr.ToString("HH:mm") },
            new { Name = "Ashar",   TimeStr = t.Asr.ToString("HH:mm") },
            new { Name = "Maghrib", TimeStr = t.Maghrib.ToString("HH:mm") },
            new { Name = "Isya",    TimeStr = t.Isha.ToString("HH:mm") }
        };
    }

    private void BtnPickAdzan_Click(object s, RoutedEventArgs e)
    {
        var dlg = new OpenFileDialog { Filter = "Audio|*.mp3;*.wav;*.aac;*.ogg" };
        if (dlg.ShowDialog() == true)
        {
            _prayer.SoundPath = dlg.FileName;
        }
    }

    // ─── Todo ────────────────────────────────────────────────
    private void BtnAddTodo_Click(object s, RoutedEventArgs e)
    {
        var dlg = new TodoDialog();
        if (dlg.ShowDialog() == true && dlg.Result != null)
            _todo.Add(dlg.Result);
    }

    private void TodoItem_Toggle(object s, RoutedEventArgs e)
    {
        if ((s as FrameworkElement)?.DataContext is TodoItem item)
            _todo.Toggle(item);
    }

    private void TodoItem_Delete(object s, RoutedEventArgs e)
    {
        if ((s as FrameworkElement)?.DataContext is TodoItem item)
            _todo.Remove(item);
    }

    // ─── Notes ───────────────────────────────────────────────
    private void BtnAddNote_Click(object s, RoutedEventArgs e)
    {
        _activeNote = new NoteItem();
        TbActiveNote.Text = "";
    }

    private void TbActiveNote_Changed(object s, TextChangedEventArgs e)
    {
        if (_activeNote == null) _activeNote = new NoteItem();
        _activeNote.Content = TbActiveNote.Text;
        _note.AddOrUpdate(_activeNote);
    }

    // ─── Clipboard ───────────────────────────────────────────
    private void ClipItem_Click(object s, MouseButtonEventArgs e)
    {
        if ((s as FrameworkElement)?.DataContext is ClipboardEntry entry)
            _clipboard.Paste(entry);
    }

    private void BtnClearClip_Click(object s, RoutedEventArgs e) => _clipboard.Clear();

    // ─── Shelf ───────────────────────────────────────────────
    private void BtnShelf_DragEnter(object s, DragEventArgs e) { e.Effects = DragDropEffects.Copy; e.Handled = true; }
    private void BtnShelf_Drop(object s, DragEventArgs e) { _shelf.HandleDrop(e.Data); ShowPanel("Shelf"); }
    private void PanelShelf_DragEnter(object s, DragEventArgs e) { e.Effects = DragDropEffects.Copy; e.Handled = true; }
    private void PanelShelf_Drop(object s, DragEventArgs e) => _shelf.HandleDrop(e.Data);

    private void ShelfItem_Open(object s, MouseButtonEventArgs e)
    {
        if ((s as FrameworkElement)?.DataContext is string path)
            _shelf.Open(path);
    }

    private void ShelfItem_Remove(object s, RoutedEventArgs e)
    {
        if ((s as FrameworkElement)?.DataContext is string path)
            _shelf.Remove(path);
    }

    // ─── Power ───────────────────────────────────────────────
    private void BtnLock_Click(object s, RoutedEventArgs e) => PowerService.LockWorkStation();
    private void BtnScreenOff_Click(object s, RoutedEventArgs e) => PowerService.TurnOffScreenNow();
    private void BtnSleep_Click(object s, RoutedEventArgs e) => PowerService.SleepPc();

    private void SliderScreenTimeout_Changed(object s, RoutedPropertyChangedEventArgs<double> e)
    {
        int min = (int)e.NewValue;
        if (TbScreenTimeout != null) TbScreenTimeout.Text = min.ToString();
        PowerService.SetScreenTimeout(min);
    }

    private void SliderSleepTimeout_Changed(object s, RoutedPropertyChangedEventArgs<double> e)
    {
        int min = (int)e.NewValue;
        if (TbSleepTimeout != null) TbSleepTimeout.Text = min.ToString();
        PowerService.SetSleepTimeout(min);
    }

    // ─── Capture ─────────────────────────────────────────────
    private void ShowCapturePanel() => ShowPanel("Capture");

    private void BtnSnapFull_Click(object s, RoutedEventArgs e)
    {
        Visibility = Visibility.Hidden;
        System.Threading.Thread.Sleep(200);
        var file = ScreenCaptureService.CaptureFullScreen();
        Visibility = Visibility.Visible;
        TbCaptureResult.Text = $"Tersimpan: {file}";
    }

    private void BtnSnapArea_Click(object s, RoutedEventArgs e)
    {
        Visibility = Visibility.Hidden;
        var overlay = new CaptureOverlay();
        overlay.OnCaptured += file =>
        {
            Dispatcher.Invoke(() =>
            {
                Visibility = Visibility.Visible;
                TbCaptureResult.Text = $"Tersimpan: {file}";
            });
        };
        overlay.Closed += (_, _) => Dispatcher.Invoke(() => Visibility = Visibility.Visible);
        overlay.ShowDialog();
    }

    private void BtnRecord_Click(object s, RoutedEventArgs e)
    {
        // ponytail: Screen Record via Windows.Media.Capture atau FFmpeg embedded
        // Saat ini membuka Xbox Game Bar (Win+Alt+R) sebagai fallback bawaan Windows
        try
        {
            System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
            {
                FileName = "ms-xbox-gamebar:",
                UseShellExecute = true
            });
        }
        catch
        {
            TbCaptureResult.Text = "Tekan Win+Alt+R untuk merekam layar.";
        }
    }

    // ─── Panel Toggle ────────────────────────────────────────
    private void BtnMedia_Click(object s, RoutedEventArgs e) => ShowPanel("Media");
    private void BtnPrayer_Click(object s, RoutedEventArgs e) => ShowPanel("Prayer");
    private void BtnTodo_Click(object s, RoutedEventArgs e) => ShowPanel("Todo");
    private void BtnNote_Click(object s, RoutedEventArgs e) => ShowPanel("Note");
    private void BtnClip_Click(object s, RoutedEventArgs e) => ShowPanel("Clipboard");
    private void BtnShelf_Click(object s, RoutedEventArgs e) => ShowPanel("Shelf");
    private void BtnPower_Click(object s, RoutedEventArgs e) => ShowPanel("Power");
    private void BtnCal_Click(object s, RoutedEventArgs e) => ShowPanel("Calendar");

    private void ShowPanel(string panelName)
    {
        // Map nama ke UI element
        var map = new Dictionary<string, Border>
        {
            { "Media",     PanelMedia },
            { "Lyrics",    PanelLyrics },
            { "Prayer",    PanelPrayer },
            { "Todo",      PanelTodo },
            { "Note",      PanelNote },
            { "Clipboard", PanelClipboard },
            { "Shelf",     PanelShelf },
            { "Power",     PanelPower },
            { "Calendar",  PanelCalendar },
            { "Capture",   PanelCapture },
        };

        // Toggle: kalau panel yang sama diklik lagi, tutup semua
        bool samePanel = _activePanelName == panelName;

        foreach (var (_, panel) in map)
            panel.Visibility = Visibility.Collapsed;

        if (samePanel)
        {
            CollapseAll();
            return;
        }

        _activePanelName = panelName;
        if (map.TryGetValue(panelName, out var target))
        {
            target.Visibility = Visibility.Visible;
            // Media player: tampilkan lirik sekalian
            if (panelName == "Media") PanelLyrics.Visibility = Visibility.Visible;
        }

        Expand();
    }

    private void Expand()
    {
        if (!_isExpanded)
        {
            _isExpanded = true;
            ExpandedPanel.Visibility = Visibility.Visible;
            BeginStoryboard((Storyboard)Resources["ExpandAnim"]);
        }
    }

    private void CollapseAll()
    {
        _activePanelName = null;
        _isExpanded = false;
        ExpandedPanel.Visibility = Visibility.Collapsed;
        BeginStoryboard((Storyboard)Resources["CollapseAnim"]);
    }

    private void CollapseToNormal()
    {
        var anim = new DoubleAnimation(36, TimeSpan.FromSeconds(0.2));
        NotchBorder.BeginAnimation(HeightProperty, anim);
    }

    // ─── Hover collapse/expand ────────────────────────────────
    private void NotchBorder_MouseEnter(object s, MouseEventArgs e)
    {
        if (!_isExpanded && NotchBorder.Height < 36)
            CollapseToNormal();
    }

    private void NotchBorder_MouseLeave(object s, MouseEventArgs e) { }

    private void NotchBorder_MouseLeftButtonDown(object s, MouseButtonEventArgs e)
    {
        if (e.ClickCount == 2) CollapseAll();
    }
}
