# Compila el instalador firmado de Noki.
#
# Uso:   powershell -ExecutionPolicy Bypass -File scripts\compilar.ps1
#
# Usa la clave privada de actualizaciones (por defecto ~\.tauri\noki.key). Sin ella el instalador se
# crea igual, pero no se genera el archivo .sig y las apps ya instaladas no podrán actualizarse con él.
# La clave NUNCA debe subirse al repositorio.
param(
    [string]$Key = ""
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

if (-not $Key) { $Key = Join-Path $env:USERPROFILE ".tauri\noki.key" }

if (-not $Key -or -not (Test-Path $Key)) {
    Write-Warning "No se encontró la clave privada de actualizaciones. Se compila sin firmar."
} else {
    $env:TAURI_SIGNING_PRIVATE_KEY = $Key
    if (-not $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD) { $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "" }
}

npx tauri build
if ($LASTEXITCODE -ne 0) { throw "La compilación falló." }

$version = (Get-Content src-tauri\tauri.conf.json -Raw | ConvertFrom-Json).version
$dir = "src-tauri\target\release\bundle\nsis"
Write-Host "`nListo:" -ForegroundColor Green
Get-ChildItem $dir -Filter "*$version*" | ForEach-Object { Write-Host "  $($_.FullName)" }
