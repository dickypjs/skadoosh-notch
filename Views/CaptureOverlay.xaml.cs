using System;
using System.Drawing;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using SkadooshNotch.Services;

namespace SkadooshNotch.Views;

public partial class CaptureOverlay : Window
{
    private System.Windows.Point _startPoint;
    private bool _isDragging;

    public event Action<string>? OnCaptured;

    public CaptureOverlay()
    {
        InitializeComponent();
    }

    private void Window_MouseDown(object sender, MouseButtonEventArgs e)
    {
        if (e.LeftButton == MouseButtonState.Pressed)
        {
            _startPoint = e.GetPosition(this);
            _isDragging = true;
            SelectionRect.Visibility = Visibility.Visible;
            Canvas.SetLeft(SelectionRect, _startPoint.X);
            Canvas.SetTop(SelectionRect, _startPoint.Y);
            SelectionRect.Width = 0;
            SelectionRect.Height = 0;
        }
    }

    private void Window_MouseMove(object sender, System.Windows.Input.MouseEventArgs e)
    {
        if (_isDragging)
        {
            var cur = e.GetPosition(this);
            double x = Math.Min(cur.X, _startPoint.X);
            double y = Math.Min(cur.Y, _startPoint.Y);
            double w = Math.Abs(cur.X - _startPoint.X);
            double h = Math.Abs(cur.Y - _startPoint.Y);

            Canvas.SetLeft(SelectionRect, x);
            Canvas.SetTop(SelectionRect, y);
            SelectionRect.Width = w;
            SelectionRect.Height = h;
        }
    }

    private void Window_MouseUp(object sender, MouseButtonEventArgs e)
    {
        if (_isDragging)
        {
            _isDragging = false;
            Hide();

            double x = Canvas.GetLeft(SelectionRect);
            double y = Canvas.GetTop(SelectionRect);
            double w = SelectionRect.Width;
            double h = SelectionRect.Height;

            if (w > 10 && h > 10)
            {
                // Convert WPF Dip ke screen pixel
                var source = PresentationSource.FromVisual(this);
                double dpiX = source?.CompositionTarget?.TransformToDevice.M11 ?? 1.0;
                double dpiY = source?.CompositionTarget?.TransformToDevice.M22 ?? 1.0;

                var rect = new Rectangle((int)(x * dpiX), (int)(y * dpiY), (int)(w * dpiX), (int)(h * dpiY));
                var file = ScreenCaptureService.CaptureArea(rect);
                OnCaptured?.Invoke(file);
            }

            Close();
        }
    }

    private void Window_KeyDown(object sender, System.Windows.Input.KeyEventArgs e)
    {
        if (e.Key == Key.Escape) Close();
    }
}
