using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace SkadooshNotch.Services;

public class WindowMonitorService
{
    [DllImport("user32.dll")]
    private static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll", SetLastError = true)]
    private static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetWindowRect(IntPtr hWnd, out RECT lpRect);

    [StructLayout(LayoutKind.Sequential)]
    private struct RECT
    {
        public int Left;
        public int Top;
        public int Right;
        public int Bottom;
    }

    private static readonly string[] _browserProcesses = new[]
    {
        "chrome", "msedge", "firefox", "brave", "opera", "vivaldi"
    };

    public bool IsBrowserActive { get; private set; }
    public bool IsFullscreenActive { get; private set; }

    public event Action<bool, bool>? OnStateChanged;

    public void CheckActiveWindow()
    {
        try
        {
            var hWnd = GetForegroundWindow();
            if (hWnd == IntPtr.Zero) return;

            GetWindowThreadProcessId(hWnd, out uint pid);
            if (pid == 0) return;

            using var proc = Process.GetProcessById((int)pid);
            var procName = proc.ProcessName.ToLowerInvariant();

            bool browser = false;
            foreach (var b in _browserProcesses)
            {
                if (procName.Contains(b)) { browser = true; break; }
            }

            // Check Fullscreen
            bool fullscreen = false;
            if (GetWindowRect(hWnd, out RECT rect))
            {
                var screen = Screen.PrimaryScreen?.Bounds ?? System.Drawing.Rectangle.Empty;
                if (screen != System.Drawing.Rectangle.Empty &&
                    rect.Left <= screen.Left && rect.Top <= screen.Top &&
                    rect.Right >= screen.Right && rect.Bottom >= screen.Bottom)
                {
                    // Pastikan bukan desktop/explorer shell
                    if (procName != "explorer")
                    {
                        fullscreen = true;
                    }
                }
            }

            if (browser != IsBrowserActive || fullscreen != IsFullscreenActive)
            {
                IsBrowserActive = browser;
                IsFullscreenActive = fullscreen;
                OnStateChanged?.Invoke(IsBrowserActive, IsFullscreenActive);
            }
        }
        catch { }
    }
}
