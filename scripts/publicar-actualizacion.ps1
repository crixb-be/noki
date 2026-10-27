# Prepara los archivos de una actualización para subirlos a tu servidor (GitHub Releases, tu hosting…).
#
# Uso:   powershell -ExecutionPolicy Bypass -File scripts\publicar-actualizacion.ps1 `
#            -BaseUrl "https://github.com/TU-USUARIO/noki/releases/download/v0.7.0" `
#            -Notas "Qué hay de nuevo en esta versión"
#
# 1. Compila antes con scripts\compilar.ps1 (genera el instalador y su firma .sig).
# 2. Este script crea la carpeta `publicar\` con:
#       - el instalador con un nombre sin espacios,
#       - latest.json, el archivo que lee la app (con la URL del instalador y su firma).
# 3. Sube AMBOS archivos a `-BaseUrl` y pon la dirección de latest.json en
#    Ajustes → Acerca de → «Dirección de las actualizaciones».
param(
    [Parameter(Mandatory = $true)][string]$BaseUrl,
    [string]$Notas = ""
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

$version = (Get-Content src-tauri\tauri.conf.json -Raw | ConvertFrom-Json).version
$bundle = "src-tauri\target\release\bundle\nsis"
$exe = Join-Path $bundle "Noki_${version}_x64-setup.exe"
$sig = "$exe.sig"
if (-not (Test-Path $exe) -or -not (Test-Path $sig)) {
    throw "Falta el instalador o su firma (.sig) de la versión $version. Ejecuta primero scripts\compilar.ps1."
}

$out = "publicar"
New-Item -ItemType Directory -Force $out | Out-Null
$name = "Noki_${version}_x64-setup.exe"
Copy-Item $exe (Join-Path $out $name) -Force   # copia idéntica: la firma sigue siendo válida

$base = $BaseUrl.TrimEnd('/')
$latest = [ordered]@{
    version  = $version
    notes    = $Notas
    pub_date = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    platforms = [ordered]@{
        "windows-x86_64" = [ordered]@{
            signature = (Get-Content $sig -Raw).Trim()
            url       = "$base/$name"
        }
    }
}
[IO.File]::WriteAllText((Join-Path $out "latest.json"), ($latest | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding($false)))

Write-Host "Listo. Sube estos dos archivos a $base :" -ForegroundColor Green
Get-ChildItem $out | ForEach-Object { Write-Host "  $($_.FullName)" }
Write-Host "Dirección para la app: $base/latest.json"
