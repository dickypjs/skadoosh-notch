using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Windows;

namespace SkadooshNotch.Services;

public class ShelfService
{
    public ObservableCollection<string> Items { get; } = new();
    private static readonly string _file = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "SkadooshNotch", "shelf.txt");

    public ShelfService()
    {
        Directory.CreateDirectory(Path.GetDirectoryName(_file)!);
        Load();
    }

    private void Load()
    {
        if (File.Exists(_file))
        {
            foreach (var line in File.ReadAllLines(_file))
                if (!string.IsNullOrWhiteSpace(line)) Items.Add(line.Trim());
        }
    }

    private void Save()
    {
        try { File.WriteAllLines(_file, Items); }
        catch { }
    }

    public void AddPath(string path)
    {
        if (!Items.Contains(path))
        {
            Items.Insert(0, path);
            Save();
        }
    }

    public void Remove(string path)
    {
        Items.Remove(path);
        Save();
    }

    public void Open(string path)
    {
        try
        {
            System.Diagnostics.Process.Start(new System.Diagnostics.ProcessStartInfo
            {
                FileName = path,
                UseShellExecute = true
            });
        }
        catch { }
    }

    public bool HandleDrop(IDataObject data)
    {
        if (data.GetDataPresent(DataFormats.FileDrop))
        {
            var files = data.GetData(DataFormats.FileDrop) as string[];
            if (files != null)
            {
                foreach (var f in files) AddPath(f);
                return true;
            }
        }
        return false;
    }
}
