namespace Demo;

public interface IRestore { Task<bool> Offer(); Task<bool> Later(); }
public class Step { public bool IsSelected { get; set; } public Step? Next { get; set; } public int Count { get; set; } }

public partial class Shell
{
    private readonly IRestore _restore;
    private readonly int[]? _slots;

    partial void OnChanged(Step? oldValue, Step? newValue)
    {
        oldValue?.IsSelected = false;
        newValue?.IsSelected = true;
        newValue?.Next?.IsSelected = true;
        newValue?.Count += 1;
        _slots?[0] = 1;
    }

    public async Task AfterAssign() { if (!await _restore.Offer()) { return; } }
    public bool Compare(Step? s) { return s?.Count == 1 && s?.IsSelected == true; }
    public async Task Last() { if (!await _restore.Later()) { return; } }
}

public partial class Panel
{
    private readonly IRestore _restore;

    partial void OnChanged(Step? oldValue, Step? newValue)
    {
        oldValue?.IsSelected = false;
        newValue?.IsSelected = true;
    }

    public async Task AfterAssign() { if (!await _restore.Offer()) { return; } }
}
