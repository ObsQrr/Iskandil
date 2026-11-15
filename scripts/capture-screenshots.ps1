$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$exe = Join-Path $root 'target\release\iskandil.exe'
if (-not (Test-Path $exe)) { throw "once release derlemesi gerekli: $exe" }
$out = Join-Path $root 'docs\screenshots'
New-Item -ItemType Directory -Force -Path $out | Out-Null
$demo = Get-ChildItem $root -Directory | Where-Object { $_.Name -like '*Demo' } | Select-Object -First 1
if (-not $demo) { throw "demo klasoru yok; scripts\make_demo.py calistir" }
$demoArg = '"' + $demo.FullName + '"'
$size = '1480x920'

function Shot([string[]]$Arguments) {
    $p = Start-Process -FilePath $exe -ArgumentList $Arguments -PassThru
    if (-not $p.WaitForExit(180000)) {
        try { $p.Kill() } catch {}
        throw "iskandil zaman asimi"
    }
    if ($p.ExitCode -ne 0) { throw "iskandil cikis kodu: $($p.ExitCode)" }
}

Shot @('--shot', (Join-Path $out '01-start-light.png'), '--theme', 'light', '--size', $size)
Shot @('--scan', $demoArg, '--shot', (Join-Path $out '02-workspace-light.png'), '--theme', 'light', '--size', $size)
Shot @('--scan', $demoArg, '--shot', (Join-Path $out '03-workspace-dark.png'), '--theme', 'dark', '--size', $size)
Shot @('--scan', 'C:\', '--shot', (Join-Path $out '04-scanning.png'), '--shot-now', '--theme', 'light', '--size', $size)
Shot @('--scan', $demoArg, '--view', 'Projeler', '--shot', (Join-Path $out '05-projects.png'), '--theme', 'light', '--size', $size)
Shot @('--scan', $demoArg, '--search', 'mp4', '--shot', (Join-Path $out '06-search.png'), '--theme', 'light', '--size', $size)
Shot @('--scan', $demoArg, '--ask-delete-largest', '--shot', (Join-Path $out '07-delete.png'), '--theme', 'light', '--size', $size)
Write-Output "ekran goruntuleri guncellendi: $out"
