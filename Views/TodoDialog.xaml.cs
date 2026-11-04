using System;
using System.Windows;
using SkadooshNotch.Models;

namespace SkadooshNotch.Views;

public partial class TodoDialog : Window
{
    public TodoItem? Result { get; private set; }

    public TodoDialog()
    {
        InitializeComponent();
        DpDeadline.SelectedDate = DateTime.Today;
        TbTitle.Focus();
    }

    private void BtnSave_Click(object sender, RoutedEventArgs e)
    {
        if (string.IsNullOrWhiteSpace(TbTitle.Text))
        {
            MessageBox.Show("Judul tugas tidak boleh kosong.", "SkadooshNotch", MessageBoxButton.OK, MessageBoxImage.Warning);
            return;
        }

        Result = new TodoItem
        {
            Title = TbTitle.Text.Trim(),
            Deadline = DpDeadline.SelectedDate,
            Done = false
        };

        DialogResult = true;
        Close();
    }

    private void BtnCancel_Click(object sender, RoutedEventArgs e)
    {
        DialogResult = false;
        Close();
    }
}
