namespace Demo;

public interface IGate
{
    Task<bool> Negated();
    Task<bool> Awaited();
    Task<bool> Plain();
    Task<bool> Grouped();
    Task<bool> Chained();
    Task<bool> Safe();
    Task<bool> Primary();
    Task<bool> Mixed();
}

public interface IFlag { bool Ready { get; } }

public class Gatekeeper
{
    private readonly IGate _gate;
    private readonly IFlag? _flag;

    public Gatekeeper(IGate gate) { _gate = gate; }

    public async Task<int> NegatedAwait() { if (!await _gate.Negated()) { return 1; } return 0; }
    public async Task<int> PlainAwait() { if (await _gate.Awaited()) { return 1; } return 0; }
    public async Task<int> NegatedCall() { if (!_gate.Plain().Result) { return 1; } return 0; }
    public async Task<int> GroupedAwait() { if (!(await _gate.Grouped())) { return 1; } return 0; }
    public async Task<int> ChainedAwait() { if (!await _gate.Chained().ConfigureAwait(false)) { return 1; } return 0; }
    public async Task<int> SafeAwait() { if (!await _gate?.Safe()) { return 1; } return 0; }
    public async Task<int> MixedAwait(bool x) { return x || !await _gate.Mixed() ? 1 : 0; }
    public bool NegatedMember() { return !_flag?.Ready ?? false; }
    public bool NegatedAccess() { return !_flag.Ready; }
}

public class PrimaryGatekeeper(IGate gate)
{
    public async Task<int> Run() { if (!await gate.Primary()) { return 1; } return 0; }
}
