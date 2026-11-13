# Acceptance of M1 and M2 against the private C# reference (ADR 0017, blueprint 19).
# Runs only on the product owner's machine. Nothing it writes belongs into a repository:
# all files go to $Out (outside the repository). `ergebnis.txt` holds numbers only, no names or paths,
# so it can be pasted into the project chat. Names and call sites stay in `pruefung-usages.txt`.
#
# Usage, from the root folder of the CodeSkimr checkout (PowerShell 5.1 or 7):
#   .\tools\acceptance\private-reference.ps1 -Src 'C:\path\to\private-reference\src'
param(
    [Parameter(Mandatory)][string]$Src,
    [string]$Out = (Join-Path $HOME 'codeskimr-abnahme-privat')
)
$ErrorActionPreference = 'Continue'
$env:CODESKIMR_LANG = 'en'
if (-not (Test-Path (Join-Path $PWD 'BLUEPRINT.md'))) { throw 'Bitte im CodeSkimr-Ordner starten.' }
if (-not (Test-Path $Src)) { throw "Ordner nicht gefunden: $Src" }
$Src = (Resolve-Path $Src).Path
New-Item -ItemType Directory -Force $Out | Out-Null
$exe = Join-Path $PWD 'target\release\codeskimr.exe'
$report = Join-Path $Out 'ergebnis.txt'
function Log([string]$t) { Write-Host $t; Add-Content -Path $report -Value $t }
function Median($v) { $s = @($v | Sort-Object); $s[[int][math]::Floor(($s.Count - 1) / 2)] }
Set-Content $report "Abnahme privat, $(Get-Date -Format s), CodeSkimr $(git rev-parse --short HEAD)"

Log '== 1. Bauen'
cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'cargo build fehlgeschlagen' }

Log '== 2. Zaehlregel ADR 0017 (zweimal messen, gegen Baseline vergleichen)'
function Measure-Counts([string]$file) {
    $env:CODESKIMR_CORPUS_DIR = $Src; $env:CODESKIMR_BLESS = '1'
    cargo test -p codeskimr_parsing --test corpus -- --nocapture 2>&1 | Out-String | Set-Content $file
    Remove-Item Env:CODESKIMR_CORPUS_DIR, Env:CODESKIMR_BLESS
    (Get-Content $file | Where-Object { $_ -match '^SUMMARY' } | Select-Object -First 1)
}
$first = Measure-Counts (Join-Path $Out 'zaehlung-1.txt')
$second = Measure-Counts (Join-Path $Out 'zaehlung-2.txt')
Log ("Zaehlung reproduzierbar (zwei Laeufe gleich): " + ($first -and $first -eq $second))
$baseline = Join-Path $Out 'baseline.txt'
if (-not (Test-Path $baseline)) { Set-Content $baseline $first; Log 'Baseline fehlte: diese Messung ist jetzt die Baseline (Version der Anwendung selbst notieren).' }
else { Log ("Zaehlung gleich der Baseline: " + ((Get-Content $baseline -Raw).Trim() -eq $first)) }

Log '== 3. Voller Scan (Budget: 100k LOC unter 1 s)'
$files = Get-ChildItem $Src -Recurse -Filter *.cs | Where-Object { $_.FullName -notmatch '\\(bin|obj)\\' -and $_.Name -notlike '*.g.cs' }
$loc = ($files | ForEach-Object { [IO.File]::ReadAllLines($_.FullName).Length } | Measure-Object -Sum).Sum
$secs = foreach ($i in 1..3) {
    $env:CODESKIMR_DATA_DIR = Join-Path $Out "db$i"
    (& $exe scan $Src --json 2>$null | ConvertFrom-Json).seconds
}
$m = Median $secs
Log ("Dateien: $($files.Count), LOC: $loc, Sekunden (3 Laeufe): $($secs -join ' / '), Median: $m, hochgerechnet auf 100k LOC: " + [math]::Round($m * 100000 / $loc, 2))

Log '== 4. Inkrementeller Scan (Budget: unter 300 ms), auf einer Kopie'
$copy = Join-Path $Out 'kopie'
robocopy $Src $copy /MIR /XD bin obj /NFL /NDL /NJH /NJS /NP | Out-Null
$env:CODESKIMR_DATA_DIR = Join-Path $Out 'db-watch'
$outFile = Join-Path $Out 'watch.out'
$watch = Start-Process $exe -ArgumentList 'scan', "`"$copy`"", '--watch' -PassThru -WindowStyle Hidden -RedirectStandardOutput $outFile -RedirectStandardError (Join-Path $Out 'watch.err')
try {
$deadline = (Get-Date).AddMinutes(10)
while ((Get-Date) -lt $deadline -and -not ((Get-Content $outFile -ErrorAction SilentlyContinue) -match 'Watching')) { Start-Sleep 1 }
$target = (Get-ChildItem $copy -Recurse -Filter *.cs | Where-Object { $_.Name -notlike '*.g.cs' -and $_.FullName -notmatch '\\(bin|obj)\\' } | Select-Object -First 1).FullName
$times = @()
foreach ($i in 1..5) {
    Add-Content $target "// abnahme $i"
    $stop = (Get-Date).AddSeconds(60)
    while ((Get-Date) -lt $stop -and @(Get-Content $outFile | Where-Object { $_ -match '\d+ ms' }).Count -lt $i) { Start-Sleep -Milliseconds 200 }
}
} finally { Stop-Process -Id $watch.Id -Force -ErrorAction SilentlyContinue }
$times = @(Get-Content $outFile | ForEach-Object { if ($_ -match '(\d+) ms') { [int]$Matches[1] } })
Log ("Inkrementelle Scans in ms: " + ($times -join ' / ') + ", Median: " + $(if ($times.Count) { Median $times } else { 'keine Messung' }))

Log '== 5. MCP und Stichprobe der Verwendungen (20 Symbole)'
$env:CODESKIMR_DATA_DIR = Join-Path $Out 'db1'
$psi = New-Object Diagnostics.ProcessStartInfo $exe, "mcp --project `"$Src`""
$psi.UseShellExecute = $false; $psi.RedirectStandardInput = $true; $psi.RedirectStandardOutput = $true
$psi.StandardOutputEncoding = New-Object Text.UTF8Encoding($false)
$mcp = [Diagnostics.Process]::Start($psi)
$writer = New-Object IO.StreamWriter($mcp.StandardInput.BaseStream, (New-Object Text.UTF8Encoding($false)))
$script:id = 0
function Rpc([string]$method, $params) {
    $script:id++
    $writer.WriteLine((@{ jsonrpc = '2.0'; id = $script:id; method = $method; params = $params } | ConvertTo-Json -Depth 8 -Compress))
    $writer.Flush()
    $mcp.StandardOutput.ReadLine() | ConvertFrom-Json
}
function Tool([string]$name, $arguments) { (Rpc 'tools/call' @{ name = $name; arguments = $arguments }).result }
$null = Rpc 'initialize' @{ protocolVersion = '2025-03-26'; capabilities = @{}; clientInfo = @{ name = 'abnahme'; version = '0' } }
$writer.WriteLine('{"jsonrpc":"2.0","method":"notifications/initialized"}'); $writer.Flush()
$tools = @((Rpc 'tools/list' @{}).result.tools | ForEach-Object { $_.name })
Log ("MCP-Werkzeuge ($($tools.Count)): " + ($tools -join ', '))

$text = @{}; foreach ($f in $files) { $text[$f.FullName] = [IO.File]::ReadAllLines($f.FullName) }
function Text-Hits([string]$name) {
    $re = [regex]('\b' + [regex]::Escape($name) + '\b')
    foreach ($f in $text.Keys) { for ($n = 0; $n -lt $text[$f].Count; $n++) { if ($re.IsMatch($text[$f][$n])) { "$($f.Substring($Src.Length)):$($n + 1)" } } }
}
$pool = @{}
foreach ($q in 'a', 'e', 'i', 'o', 'u') {
    foreach ($kind in 'method', 'constant', 'property') {
        foreach ($h in (Tool 'search_symbols' @{ query = $q; kind = $kind; limit = 100 }).structuredContent.hits) { if ($h.name.Length -ge 5) { $pool[$h.id] = $h } }
    }
}
$picked = @()
foreach ($group in @(@('method'), @('constant', 'property'))) {
    $all = @($pool.Values | Where-Object { $group -contains $_.kind } | Sort-Object id)
    $stride = [math]::Max(1, [int][math]::Floor($all.Count / 60))
    $sampled = @(for ($k = 0; $k -lt $all.Count; $k += $stride) { $all[$k] })
    $c = @($sampled | Where-Object { $n = @(Text-Hits $_.name).Count; $n -ge 2 -and $n -le 30 })
    $step = [math]::Max(1, [int][math]::Floor($c.Count / 10))
    for ($k = 0; $k -lt $c.Count -and $k -lt 10 * $step; $k += $step) { $picked += $c[$k] }
}
$review = Join-Path $Out 'pruefung-usages.txt'
Set-Content $review "Je Symbol: Textsuche (Fundstellen des Namens) und Verwendungen laut CodeSkimr. Beurteilen: stimmt jede genannte Verwendung, und fehlt keine echte?"
$rows = @()
foreach ($s in $picked) {
    $timer = [Diagnostics.Stopwatch]::StartNew()
    $u = (Tool 'find_usages' @{ symbol = $s.qualified_name; limit = 100 }).structuredContent
    $timer.Stop()
    $certain = @($u.usages | Where-Object { $_.confidence -eq 'certain' }).Count
    $rows += [pscustomobject]@{ Total = $u.total; Certain = $certain; Ms = $timer.Elapsed.TotalMilliseconds }
    Add-Content $review ("`n## [$($s.kind)] $($s.qualified_name)`nTextsuche: " + (@(Text-Hits $s.name) -join ' ') + "`nCodeSkimr ($($u.total), davon certain $certain): " + (@($u.usages | ForEach-Object { "$($_.path):$($_.line)" }) -join ' '))
}
$writer.Close(); $mcp.WaitForExit(5000) | Out-Null
Log ("Stichprobe: $($rows.Count) Symbole, Verwendungen gesamt $(($rows | Measure-Object Total -Sum).Sum), davon certain $(($rows | Measure-Object Certain -Sum).Sum), langsamste Abfrage $([math]::Round(($rows | Measure-Object Ms -Maximum).Maximum, 1)) ms")
Log "Zur Beurteilung: pruefung-usages.txt im Ausgabeordner (bleibt lokal)."
Write-Host "`nFertig. Bitte den Inhalt von $report in den Projekt-Chat kopieren und sagen, wie viele der 20 Symbole in $review stimmen."
