using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Text.Json;
using System.Windows;
using SkadooshNotch.Models;

namespace SkadooshNotch.Services;

public class ClipboardService
{
    public ObservableCollection<ClipboardEntry> History { get; } = new();
    private const int MaxHistory = 30;
    private string? _lastText;

    public event Action? OnClipboardChanged;

    public void PollClipboard()
    {
        try
        {
            if (Clipboard.ContainsText())
            {
                var text = Clipboard.GetText();
                if (text != _lastText && !string.IsNullOrWhiteSpace(text))
                {
                    _lastText = text;
                    var entry = new ClipboardEntry { Text = text[..Math.Min(text.Length, 500)] };
                    History.Insert(0, entry);
                    if (History.Count > MaxHistory) History.RemoveAt(History.Count - 1);
                    OnClipboardChanged?.Invoke();
                }
            }
        }
        catch { }
    }

    public void Paste(ClipboardEntry entry)
    {
        try { Clipboard.SetText(entry.Text); }
        catch { }
    }

    public void Remove(ClipboardEntry entry) => History.Remove(entry);
    public void Clear() => History.Clear();
}

public class TodoService
{
    private static readonly string _file = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "SkadooshNotch", "todos.json");

    public ObservableCollection<TodoItem> Items { get; } = new();

    public TodoService()
    {
        Directory.CreateDirectory(Path.GetDirectoryName(_file)!);
        Load();
    }

    private void Load()
    {
        try
        {
            if (File.Exists(_file))
            {
                var list = JsonSerializer.Deserialize<TodoItem[]>(File.ReadAllText(_file)) ?? Array.Empty<TodoItem>();
                foreach (var i in list) Items.Add(i);
            }
        }
        catch { }
    }

    public void Save()
    {
        try { File.WriteAllText(_file, JsonSerializer.Serialize(Items)); }
        catch { }
    }

    public void Add(TodoItem item) { Items.Add(item); Save(); }
    public void Remove(TodoItem item) { Items.Remove(item); Save(); }
    public void Toggle(TodoItem item) { item.Done = !item.Done; Save(); }
}

public class NoteService
{
    private static readonly string _file = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "SkadooshNotch", "notes.json");

    public ObservableCollection<NoteItem> Items { get; } = new();

    public NoteService()
    {
        Directory.CreateDirectory(Path.GetDirectoryName(_file)!);
        Load();
    }

    private void Load()
    {
        try
        {
            if (File.Exists(_file))
            {
                var list = JsonSerializer.Deserialize<NoteItem[]>(File.ReadAllText(_file)) ?? Array.Empty<NoteItem>();
                foreach (var i in list) Items.Add(i);
            }
        }
        catch { }
    }

    public void Save()
    {
        try { File.WriteAllText(_file, JsonSerializer.Serialize(Items)); }
        catch { }
    }

    public void AddOrUpdate(NoteItem item)
    {
        var existing = Items.FirstOrDefault(n => n.Id == item.Id);
        if (existing != null) { existing.Content = item.Content; existing.UpdatedAt = DateTime.Now; }
        else Items.Add(item);
        Save();
    }

    public void Remove(NoteItem item) { Items.Remove(item); Save(); }

    // LINQ FirstOrDefault — NoteService is internal, safe to use
    private NoteItem? FirstOrDefault(Func<NoteItem, bool> pred)
    {
        foreach (var i in Items) if (pred(i)) return i;
        return null;
    }
}
