# Heico - Desinstallation
#
# Retire la cle registre du menu contextuel et supprime %LOCALAPPDATA%\Heico\.

$ErrorActionPreference = "Continue"

$supportedExts = @(
    ".heic", ".HEIC", ".heif", ".HEIF",
    ".png", ".PNG",
    ".webp", ".WEBP",
    ".tif", ".TIF", ".tiff", ".TIFF",
    ".bmp", ".BMP",
    ".gif", ".GIF",
    ".avif", ".AVIF"
)
foreach ($ext in $supportedExts) {
    $keyBase = "HKCU:\Software\Classes\SystemFileAssociations\$ext\shell\HeicoConvertToJpg"
    if (Test-Path $keyBase) {
        Remove-Item $keyBase -Recurse -Force
        Write-Host "Cle registre retiree : $ext" -ForegroundColor Green
    }
}

$installDir = Join-Path $env:LOCALAPPDATA "Heico"
if (Test-Path $installDir) {
    Remove-Item $installDir -Recurse -Force
    Write-Host "Dossier supprime : $installDir" -ForegroundColor Green
}

Write-Host ""
Write-Host "Desinstallation terminee." -ForegroundColor Cyan
Write-Host ""
Read-Host "Appuie sur Entree pour fermer"
