# Heico - Desinstallation
#
# Retire la cle registre du menu contextuel et supprime %LOCALAPPDATA%\Heico\.

$ErrorActionPreference = "Continue"

foreach ($ext in @(".heic", ".HEIC", ".heif", ".HEIF", ".png", ".PNG")) {
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
