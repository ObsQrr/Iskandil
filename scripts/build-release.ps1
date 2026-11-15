$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root

$env:RUSTFLAGS = '-C target-feature=+crt-static'
cmd /c "cargo build -p iskandil-app --release > build-release.log 2>&1"
if ($LASTEXITCODE -ne 0) {
    Get-Content build-release.log -Tail 40
    throw "release build failed"
}

$exe = Join-Path $root 'target\release\iskandil.exe'
if (-not (Test-Path $exe)) { throw "missing exe: $exe" }

$dist = Join-Path $root 'dist\iskandil'
New-Item -ItemType Directory -Force -Path $dist | Out-Null
Copy-Item $exe (Join-Path $dist 'iskandil.exe') -Force
Copy-Item (Join-Path $root 'README.md') $dist -Force
Copy-Item (Join-Path $root 'LICENSE') $dist -Force

$zip = Join-Path $root 'dist\iskandil-0.1.0-win-x64.zip'
Compress-Archive -Path (Join-Path $dist '*') -DestinationPath $zip -Force

$size = (Get-Item $exe).Length
$zipSize = (Get-Item $zip).Length
$hash = (Get-FileHash $exe -Algorithm SHA256).Hash
$zipHash = (Get-FileHash $zip -Algorithm SHA256).Hash
Write-Output "exe: $size bytes sha256=$hash"
Write-Output "zip: $zipSize bytes sha256=$zipHash"
