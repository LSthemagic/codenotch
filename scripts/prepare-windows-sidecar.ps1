param(
  [ValidateSet('debug','release')]
  [string]$Profile = 'debug'
)

$ErrorActionPreference = 'Stop'
$hostTriple = (rustc -vV | Select-String '^host: ' | ForEach-Object { $_.Line.Substring(6) }).Trim()
if (-not $hostTriple.EndsWith('-pc-windows-msvc')) {
  throw "unsupported Windows sidecar host: $hostTriple"
}

if ($Profile -eq 'release') {
  cargo build -p nyrva-hook --release --locked
  $source = 'target/release/nyrva-hook.exe'
} else {
  cargo build -p nyrva-hook --locked
  $source = 'target/debug/nyrva-hook.exe'
}

$destinationDir = 'nyrva/binaries'
$destination = Join-Path $destinationDir "nyrva-hook-$hostTriple.exe"
New-Item -ItemType Directory -Force -Path $destinationDir | Out-Null
Copy-Item -Force $source $destination
Write-Host "prepared $destination from $source"
