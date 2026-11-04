using System;
using System.Drawing;
using System.Drawing.Imaging;
using System.IO;
using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Forms;
using System.Windows.Interop;

namespace SkadooshNotch.Services;

public class HotKeyService : IDisposable
{
    private const int WM_HOTKEY = 0x0312;
    private const uint MOD_ALT = 0x0001;
    private const uint MOD_CONTROL = 0x0002;
    private const uint MOD_SHIFT = 0x0004;

    [DllImport("user32.dll")]
    private static extern bool RegisterHotKey(IntPtr hWnd, int id, uint fsModifiers, uint vk);

    [DllImport("user32.dll")]
    private static extern bool UnregisterHotKey(IntPtr hWnd, int id);

    private IntPtr _hWnd;
    private HwndSource? _source;
    private const int HOTKEY_ID = 9001;

    public event Action? OnHotKeyPressed;

    public void Register(Window window)
    {
        var helper = new WindowInteropHelper(window);
        _hWnd = helper.Handle;
        _source = HwndSource.FromHwnd(_hWnd);
        _source?.AddHook(HwndHook);

        // Ctrl + Shift + R (0x52)
        RegisterHotKey(_hWnd, HOTKEY_ID, MOD_CONTROL | MOD_SHIFT, 0x52);
    }

    private IntPtr HwndHook(IntPtr hwnd, int msg, IntPtr wParam, IntPtr lParam, ref bool handled)
    {
        if (msg == WM_HOTKEY && wParam.ToInt32() == HOTKEY_ID)
        {
            OnHotKeyPressed?.Invoke();
            handled = true;
        }
        return IntPtr.Zero;
    }

    public void Dispose()
    {
        if (_hWnd != IntPtr.Zero)
        {
            UnregisterHotKey(_hWnd, HOTKEY_ID);
            _source?.RemoveHook(HwndHook);
        }
    }
}

public static class ScreenCaptureService
{
    public static string GetSaveDir()
    {
        var path = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.MyPictures), "SkadooshNotch");
        Directory.CreateDirectory(path);
        return path;
    }

    public static string CaptureFullScreen()
    {
        var screen = Screen.PrimaryScreen?.Bounds ?? new Rectangle(0, 0, 1920, 1080);
        using var bmp = new Bitmap(screen.Width, screen.Height);
        using (var g = Graphics.FromImage(bmp))
        {
            g.CopyFromScreen(screen.Location, System.Drawing.Point.Empty, screen.Size);
        }

        var fileName = Path.Combine(GetSaveDir(), $"Screenshot_{DateTime.Now:yyyyMMdd_HHmmss}.png");
        bmp.Save(fileName, ImageFormat.Png);

        try
        {
            System.Windows.Clipboard.SetImage(System.Windows.Interop.Imaging.CreateBitmapSourceFromHBitmap(
                bmp.GetHbitmap(), IntPtr.Zero, Int32Rect.Empty, System.Windows.Media.Imaging.BitmapSizeOptions.FromEmptyOptions()));
        }
        catch { }

        return fileName;
    }

    public static string CaptureArea(Rectangle rect)
    {
        if (rect.Width <= 0 || rect.Height <= 0) return "";

        using var bmp = new Bitmap(rect.Width, rect.Height);
        using (var g = Graphics.FromImage(bmp))
        {
            g.CopyFromScreen(rect.Location, System.Drawing.Point.Empty, rect.Size);
        }

        var fileName = Path.Combine(GetSaveDir(), $"Snip_{DateTime.Now:yyyyMMdd_HHmmss}.png");
        bmp.Save(fileName, ImageFormat.Png);

        try
        {
            System.Windows.Clipboard.SetImage(System.Windows.Interop.Imaging.CreateBitmapSourceFromHBitmap(
                bmp.GetHbitmap(), IntPtr.Zero, Int32Rect.Empty, System.Windows.Media.Imaging.BitmapSizeOptions.FromEmptyOptions()));
        }
        catch { }

        return fileName;
    }
}
