using System;

namespace SkadooshNotch.Models;

public class PrayerTime
{
    public DateTime Fajr    { get; set; }
    public DateTime Dhuhr   { get; set; }
    public DateTime Asr     { get; set; }
    public DateTime Maghrib { get; set; }
    public DateTime Isha    { get; set; }

    public (string Name, DateTime Time) Next()
    {
        var now = DateTime.Now;
        if (now < Fajr)    return ("Subuh",   Fajr);
        if (now < Dhuhr)   return ("Dzuhur",  Dhuhr);
        if (now < Asr)     return ("Ashar",   Asr);
        if (now < Maghrib) return ("Maghrib", Maghrib);
        if (now < Isha)    return ("Isya",    Isha);
        return ("Subuh", Fajr.AddDays(1));
    }
}

public class TodoItem
{
    public string   Id       { get; set; } = Guid.NewGuid().ToString();
    public string   Title    { get; set; } = "";
    public DateTime? Deadline { get; set; }
    public bool     Done     { get; set; }
    public string DeadlineStr => Deadline.HasValue ? Deadline.Value.ToString("dd/MM") : "";
}

public class NoteItem
{
    public string Id      { get; set; } = Guid.NewGuid().ToString();
    public string Content { get; set; } = "";
    public DateTime UpdatedAt { get; set; } = DateTime.Now;
}

public class ClipboardEntry
{
    public string   Id        { get; set; } = Guid.NewGuid().ToString();
    public string   Text      { get; set; } = "";
    public DateTime CopiedAt  { get; set; } = DateTime.Now;
    public bool     IsImage   { get; set; }
}
