using System;
using System.Diagnostics;
using System.Runtime.InteropServices;

namespace SkadooshNotch.Services;

public static class PowerService
{
    private const int HWND_BROADCAST = 0xFFFF;
    private const int WM_SYSCOMMAND = 0x0112;
    private const int SC_MONITORPOWER = 0xF170;

    [DllImport("user32.dll")]
    private static extern IntPtr SendMessage(IntPtr hWnd, int msg, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll")]
    public static extern void LockWorkStation();

    [DllImport("Powrprof.dll", SetLastError = true)]
    private static extern bool SetSuspendState(bool hibernate, bool forceCritical, bool disableWakeEvent);

    public static void TurnOffScreenNow()
    {
        // 2 = Monitor Off, 1 = Standby, -1 = On
        SendMessage(new IntPtr(HWND_BROADCAST), WM_SYSCOMMAND, new IntPtr(SC_MONITORPOWER), new IntPtr(2));
    }

    public static void SleepPc()
    {
        SetSuspendState(false, true, true);
    }

    public static void SetScreenTimeout(int minutes)
    {
        RunPowerCfg($"/change monitor-timeout-ac {minutes}");
        RunPowerCfg($"/change monitor-timeout-dc {minutes}");
    }

    public static void SetSleepTimeout(int minutes)
    {
        RunPowerCfg($"/change standby-timeout-ac {minutes}");
        RunPowerCfg($"/change standby-timeout-dc {minutes}");
    }

    private static void RunPowerCfg(string args)
    {
        try
        {
            var psi = new ProcessStartInfo("powercfg.exe", args)
            {
                CreateNoWindow = true,
                UseShellExecute = false
            };
            Process.Start(psi);
        }
        catch { }
    }
}
