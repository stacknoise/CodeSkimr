namespace Demo;

public interface ISave { Task<bool> Offer(); }

public partial class Model
{
    private readonly ISave _save;
    public Model(ISave save) { _save = save; Loaded += async (s, e) => { if (!await _save.Offer()) { } }; }
    public event EventHandler? Loaded;
    public ICommand Cmd => new Relay(async () => { if (!await _save.Offer()) { } });
    public bool Prop { get { return !_save.Offer().Result; } }
    public async void OnClick(object s, EventArgs e) { if (!await _save.Offer()) { return; } }
    public async Task Local() { async Task Inner() { if (!await _save.Offer()) { } } await Inner(); }
    public static Func<ISave, Task<bool>> Make = async x => !await x.Offer();
    partial void Hook();
    public async Task Free() { if (!await Helper()) { } }
}
public partial class Model
{
    private Task<bool> Helper() => Task.FromResult(true);
    partial void Hook() { }
}
