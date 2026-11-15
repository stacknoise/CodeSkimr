# Acceptance of M6 against a PRIVATE Git repository (blueprint 19: "private repository scanned without the
# secret ever appearing outside the loan scope"). Runs only on the product owner's machine.
# Nothing it writes belongs into a repository: all files go to $Out (outside the repository).
# `ergebnis-m6.txt` holds numbers and yes/no answers only, no names, paths or addresses, so it can be pasted
# into the project chat. The secret is read from a hidden prompt, kept in memory for the search only, and
# never written, printed or passed on a command line.
#
# Before: store the login of the repository once, for example
#   codeskimr credential login work --provider github --client-id <id>     (OAuth device flow)
# or  codeskimr credential set work --user <name>                          (token from the hidden prompt)
#
# Usage, from the root folder of the CodeSkimr checkout (PowerShell 5.1 or 7):
#   .\tools\acceptance\m6-private.ps1 -Url 'https://github.com/owner/private-repo.git' -Credential work [-Branch main]
param(
    [Parameter(Mandatory)][string]$Url,
    [Parameter(Mandatory)][string]$Credential,
    [string]$Branch = 'main',
    [string]$Out = (Join-Path $HOME 'codeskimr-abnahme-m6')
)
$ErrorActionPreference = 'Continue'
$env:CODESKIMR_LANG = 'en'
if (-not (Test-Path (Join-Path $PWD 'BLUEPRINT.md'))) { throw 'Bitte im CodeSkimr-Ordner starten.' }
$exe = Join-Path $PWD 'target\release\codeskimr.exe'
New-Item -ItemType Directory -Force $Out | Out-Null
$data = Join-Path $Out 'data'
Remove-Item -Recurse -Force $data -ErrorAction SilentlyContinue
$report = Join-Path $Out 'ergebnis-m6.txt'
function Log([string]$t) { Write-Host $t; Add-Content -Path $report -Value $t }
Set-Content $report "Abnahme M6 privat, $(Get-Date -Format s), CodeSkimr $(git rev-parse --short HEAD), $([Environment]::OSVersion.VersionString)"

Log '== 1. Bauen'
cargo build --release
if ($LASTEXITCODE -ne 0) { throw 'cargo build fehlgeschlagen' }

Log '== 2. Geheimnis fuer die Suche (bleibt im Speicher, wird nicht ausgegeben)'
$secure = Read-Host -AsSecureString 'Token oder Passwort des gespeicherten Logins (nur fuer die Suche)'
$bstr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
$secret = [Runtime.InteropServices.Marshal]::PtrToStringBSTR($bstr)
[Runtime.InteropServices.Marshal]::ZeroFreeBSTR($bstr)
if ([string]::IsNullOrEmpty($secret)) { throw 'Leeres Geheimnis.' }

$script:runs = 0
function Run-App([string[]]$arguments) {
    $script:runs++
    $log = Join-Path $Out "lauf-$($script:runs).txt"
    $watch = [Diagnostics.Stopwatch]::StartNew()
    $env:CODESKIMR_LOG = 'trace'
    & $exe --data-dir $data @arguments *> $log
    $code = $LASTEXITCODE
    Remove-Item Env:CODESKIMR_LOG
    $watch.Stop()
    [pscustomobject]@{ Code = $code; Ms = [int]$watch.Elapsed.TotalMilliseconds; Text = (Get-Content $log -Raw); Log = $log }
}

# Searches a file for the secret as UTF-8 and as UTF-16, without printing anything of it.
function Has-Secret([string]$file, [string[]]$needles) {
    $bytes = [IO.File]::ReadAllBytes($file)
    $latin = [Text.Encoding]::GetEncoding(28591).GetString($bytes)
    foreach ($n in $needles) { if ($latin.Contains($n)) { return $true } }
    $false
}
$needles = @(
    [Text.Encoding]::GetEncoding(28591).GetString([Text.Encoding]::UTF8.GetBytes($secret)),
    [Text.Encoding]::GetEncoding(28591).GetString([Text.Encoding]::Unicode.GetBytes($secret))
)

Log '== 3. Erster Scan mit Login (klonen und scannen)'
$first = Run-App @('scan', $Url, '--branch', $Branch, '--credential', $Credential, '--json')
$summary = $first.Text -split "`n" | Where-Object { $_ -match '^\{' } | Select-Object -First 1 | ConvertFrom-Json -ErrorAction SilentlyContinue
Log ("Exit: $($first.Code), Dauer gesamt: $($first.Ms) ms, Dateien: $($summary.files), Symbole: $($summary.symbols), Verweise: $($summary.references)")

Log '== 4. Zweiter Scan (Update, nichts Neues erwartet)'
$second = Run-App @('scan', $Url, '--branch', $Branch, '--credential', $Credential, '--json')
Log ("Exit: $($second.Code), Dauer gesamt: $($second.Ms) ms")

Log '== 5. Export'
$page = Join-Path $Out 'export.html'
$export = Run-App @('export', $Url, '--html', $page)
Log ("Exit: $($export.Code), Seite vorhanden: $(Test-Path $page)")

Log '== 6. Suche nach dem Geheimnis (Anzahl Treffer, 0 ist das Ziel)'
$outputs = @($first, $second, $export)
Log ("Ausgaben der Befehle mit Geheimnis: " + @($outputs | Where-Object { Has-Secret $_.Log $needles }).Count + " von $($outputs.Count) (Log-Stufe trace)")
Log ("Seite des Exports mit Geheimnis: $(Has-Secret $page $needles)")
$files = @(Get-ChildItem $data -Recurse -File -Force)
$cache = @($files | Where-Object { $_.FullName -match '\\git\\' })
$other = @($files | Where-Object { $_.FullName -notmatch '\\git\\' })
Log ("Dateien im Datenordner (ohne Git-Cache) mit Geheimnis: " + @($other | Where-Object { Has-Secret $_.FullName $needles }).Count + " von $($other.Count)")
Log ("Dateien im Git-Cache mit Geheimnis: " + @($cache | Where-Object { Has-Secret $_.FullName $needles }).Count + " von $($cache.Count)")
$configs = @($cache | Where-Object { $_.Name -eq 'config' })
Log ("Git-Konfigurationen im Cache mit Zugangsdaten in einer Adresse (user:pass@): " + @($configs | Where-Object { (Get-Content $_.FullName -Raw) -match '://[^/\s@]+@' }).Count + " von $($configs.Count)")
$patterns = 'ghp_[A-Za-z0-9]{20,}', 'github_pat_[A-Za-z0-9_]{20,}', 'gho_[A-Za-z0-9]{20,}', 'glpat-[A-Za-z0-9_-]{10,}'
$hits = 0
foreach ($path in (@($other | ForEach-Object { $_.FullName }) + @($first.Log, $second.Log, $export.Log, $page))) { $t = [Text.Encoding]::GetEncoding(28591).GetString([IO.File]::ReadAllBytes($path)); foreach ($p in $patterns) { if ($t -match $p) { $hits++ } } }
Log ("Dateien (ohne Git-Cache) mit einem Token-Muster (ghp_, github_pat_, gho_, glpat-): $hits")
$secret = $null; $needles = $null; [GC]::Collect()

Log '== 7. Ohne Login muss der Scan scheitern, nicht fragen'
$none = Run-App @('scan', $Url, '--branch', $Branch, '--json')
Log ("Exit ohne Login: $($none.Code) (erwartet: ungleich 0 mit SRC-401, ohne Rueckfrage im Terminal oder Anmeldefenster; kann auch 0 sein, wenn der Git-Credential-Helper des Systems das Repository oeffnet)")
Log "Fertig. Ergebnis: $report"
