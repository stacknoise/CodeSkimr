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
# Quotes one argument for the command line that Start-Process builds.
function Quote-Arg([string]$a) { if ($a -match '[\s"]' -or $a -eq '') { '"' + ($a -replace '"', '\"') + '"' } else { $a } }
# Runs the program with stdout and stderr in separate files (PowerShell 5.1 would wrap and break long
# lines such as the JSON summary when it merges them). `lauf-N.txt` holds both, for the search.
function Run-App([string[]]$arguments, [hashtable]$extraEnv = @{}) {
    $script:runs++
    $log = Join-Path $Out "lauf-$($script:runs).txt"
    $outFile = "$log.out"; $errFile = "$log.err"
    $all = @('--data-dir', $data) + $arguments
    $argLine = ($all | ForEach-Object { Quote-Arg $_ }) -join ' '
    $watch = [Diagnostics.Stopwatch]::StartNew()
    try {
        $env:CODESKIMR_LOG = 'trace'
        foreach ($k in $extraEnv.Keys) { Set-Item -Path "Env:$k" -Value $extraEnv[$k] }
        $p = Start-Process -FilePath $exe -ArgumentList $argLine -NoNewWindow -Wait -PassThru -RedirectStandardOutput $outFile -RedirectStandardError $errFile
    } finally {
        Remove-Item Env:CODESKIMR_LOG -ErrorAction SilentlyContinue
        foreach ($k in $extraEnv.Keys) { Remove-Item "Env:$k" -ErrorAction SilentlyContinue }
    }
    $watch.Stop()
    $outBytes = [IO.File]::ReadAllBytes($outFile); $errBytes = [IO.File]::ReadAllBytes($errFile)
    $fs = [IO.File]::Create($log)
    try { $fs.Write($outBytes, 0, $outBytes.Length); $fs.Write($errBytes, 0, $errBytes.Length) } finally { $fs.Dispose() }
    Remove-Item $outFile, $errFile
    $outText = [Text.Encoding]::UTF8.GetString($outBytes); $errText = [Text.Encoding]::UTF8.GetString($errBytes)
    [pscustomobject]@{ Code = $p.ExitCode; Ms = [int]$watch.Elapsed.TotalMilliseconds; Out = $outText; Text = ($outText + "`n" + $errText); Log = $log }
}

# Searches a file for the secret as UTF-8 and as UTF-16, without printing anything of it.
function Has-Secret([string]$file, [string[]]$needles) {
    $bytes = [IO.File]::ReadAllBytes($file)
    $latin = [Text.Encoding]::GetEncoding(28591).GetString($bytes)
    foreach ($n in $needles) { if ($latin.Contains($n)) { return $true } }
    $false
}
# After a failed run: the lines `error[CODE]: message` of the run (the message is the fixed text of the code, no
# names or paths), so that the report says why. Other output is only counted, never copied, because the trace
# log can hold paths and names. Nothing is shown if the output holds the secret.
function Log-Failure($run) {
    if ($run.Code -eq 0) { return }
    if ($null -ne $needles -and (Has-Secret $run.Log $needles)) { Log '  Fehlerausgabe enthaelt das Geheimnis, nicht gezeigt (das ist selbst ein Befund)'; return }
    $all = @($run.Text -split "`r?`n" | Where-Object { $_.Trim() })
    $errors = @($all | Where-Object { $_ -match '^error\[[A-Z]{3}-\d{3}\]: [^/\\:]*$' } | Select-Object -First 5)
    foreach ($l in $errors) { Log ('  > ' + $l.Trim()) }
    if ($errors.Count -eq 0) { Log "  Keine error[...]-Zeile, $($all.Count) Zeilen Ausgabe in $(Split-Path $run.Log -Leaf)" }
}
$needles = @(
    [Text.Encoding]::GetEncoding(28591).GetString([Text.Encoding]::UTF8.GetBytes($secret)),
    [Text.Encoding]::GetEncoding(28591).GetString([Text.Encoding]::Unicode.GetBytes($secret))
)

Log '== 3. Erster Scan mit Login (klonen und scannen)'
$first = Run-App @('scan', $Url, '--branch', $Branch, '--credential', $Credential, '--json')
$summary = $first.Out -split "`r?`n" | Where-Object { $_ -match '^\s*\{' } | Select-Object -First 1 | ConvertFrom-Json -ErrorAction SilentlyContinue
Log ("Exit: $($first.Code), Dauer gesamt: $($first.Ms) ms, Dateien: $($summary.files), Symbole: $($summary.symbols), Verweise: $($summary.references)")
Log-Failure $first

Log '== 4. Zweiter Scan (Update, nichts Neues erwartet)'
$second = Run-App @('scan', $Url, '--branch', $Branch, '--credential', $Credential, '--json')
Log ("Exit: $($second.Code), Dauer gesamt: $($second.Ms) ms")
Log-Failure $second

Log '== 5. Export'
$page = Join-Path $Out 'export.html'
$export = Run-App @('export', $Url, '--html', $page)
Log ("Exit: $($export.Code), Seite vorhanden: $(Test-Path $page)")
Log-Failure $export

Log '== 6. Suche nach dem Geheimnis (Anzahl Treffer, 0 ist das Ziel)'
$outputs = @($first, $second, $export)
Log ("Ausgaben der Befehle mit Geheimnis: " + @($outputs | Where-Object { Has-Secret $_.Log $needles }).Count + " von $($outputs.Count) (Log-Stufe trace)")
Log ("Seite des Exports mit Geheimnis: $(Has-Secret $page $needles)")
$files = @(Get-ChildItem $data -Recurse -File -Force)
$gitRoot = (Join-Path $data 'git') + '\'
$cache = @($files | Where-Object { $_.FullName.StartsWith($gitRoot, [StringComparison]::OrdinalIgnoreCase) })
$other = @($files | Where-Object { -not $_.FullName.StartsWith($gitRoot, [StringComparison]::OrdinalIgnoreCase) })
Log ("Dateien im Datenordner (ohne Git-Cache) mit Geheimnis: " + @($other | Where-Object { Has-Secret $_.FullName $needles }).Count + " von $($other.Count)")
Log ("Dateien im Git-Cache mit Geheimnis: " + @($cache | Where-Object { Has-Secret $_.FullName $needles }).Count + " von $($cache.Count)")
$configs = @($cache | Where-Object { $_.Name -eq 'config' })
Log ("Git-Konfigurationen im Cache mit Zugangsdaten in einer Adresse (user:pass@): " + @($configs | Where-Object { (Get-Content $_.FullName -Raw) -match '://[^/\s@]+@' }).Count + " von $($configs.Count)")
$patterns = 'ghp_[A-Za-z0-9]{20,}', 'github_pat_[A-Za-z0-9_]{20,}', 'gho_[A-Za-z0-9]{20,}', 'glpat-[A-Za-z0-9_-]{10,}'
$hits = 0
foreach ($path in (@($other | ForEach-Object { $_.FullName }) + @($first.Log, $second.Log, $export.Log, $page))) { $t = [Text.Encoding]::GetEncoding(28591).GetString([IO.File]::ReadAllBytes($path)); foreach ($p in $patterns) { if ($t -match $p) { $hits++ } } }
Log ("Dateien (ohne Git-Cache) mit einem Token-Muster (ghp_, github_pat_, gho_, glpat-): $hits")

Log '== 7. Ohne Login muss der Scan scheitern, nicht fragen'
# Step 7 runs without the login, so no secret is involved. The Git credential helpers of the system (for example
# Git Credential Manager) are switched off for the first run, so that only CodeSkimr's own login could open the
# repository; without it the scan must fail. The global file holds an empty `helper =`, which resets the list of helpers
# of the lower levels.
$emptyConfig = Join-Path $Out 'leere-gitconfig'; Set-Content $emptyConfig "[credential]`n`thelper =`n"
$noHelper = @{ GIT_CONFIG_NOSYSTEM = '1'; GIT_CONFIG_GLOBAL = $emptyConfig; GIT_TERMINAL_PROMPT = '0'; GCM_INTERACTIVE = 'never' }
$none = Run-App @('scan', $Url, '--branch', $Branch, '--json') $noHelper
Log ("Exit ohne Login und ohne Git-Helper des Systems: $($none.Code) -> " + $(if ($none.Code -ne 0) { 'OK (scheitert, keine Rueckfrage)' } else { 'NICHT ERFUELLT (Scan ohne Login gelang)' }))
Log-Failure $none
$withHelper = Run-App @('scan', $Url, '--branch', $Branch, '--json')
Log ("Exit ohne Login, mit Git-Helper des Systems: $($withHelper.Code) (0 ist zulaessig: der Helper des Systems meldet dann an, Blueprint 12)")
Log-Failure $withHelper
$secret = $null; $needles = $null; [GC]::Collect()
Log "Fertig. Ergebnis: $report"
