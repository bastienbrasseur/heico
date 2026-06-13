# Heico - Installation du menu contextuel Windows
#
# Copie heico.exe dans %LOCALAPPDATA%\Heico\ et ajoute "Convertir en JPG"
# au menu contextuel des fichiers .heic / .HEIC pour l'utilisateur courant.
#
# Aucun droit administrateur requis.

$ErrorActionPreference = "Stop"

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$exeSource = Join-Path $scriptDir "..\target\release\heico.exe"

if (-not (Test-Path $exeSource)) {
    Write-Host "heico.exe introuvable a $exeSource" -ForegroundColor Red
    Write-Host "Lance d'abord : cargo build --release" -ForegroundColor Yellow
    exit 1
}

$installDir = Join-Path $env:LOCALAPPDATA "Heico"
$exeDest = Join-Path $installDir "heico.exe"

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

Copy-Item $exeSource $exeDest -Force
Write-Host "Copie : $exeDest" -ForegroundColor Green

# Copie les DLL natives a cote si presentes (libheif, libde265, etc.)
Get-ChildItem (Split-Path -Parent $exeSource) -Filter "*.dll" -ErrorAction SilentlyContinue | ForEach-Object {
    Copy-Item $_.FullName (Join-Path $installDir $_.Name) -Force
    Write-Host "  + $($_.Name)" -ForegroundColor DarkGray
}

# Cle registre pour .heic et .HEIC (Windows traite les deux separement parfois)
foreach ($ext in @(".heic", ".HEIC", ".heif", ".HEIF")) {
    $keyBase = "HKCU:\Software\Classes\SystemFileAssociations\$ext\shell\HeicoConvertToJpg"
    New-Item -Path $keyBase -Force | Out-Null
    Set-ItemProperty -Path $keyBase -Name "(Default)" -Value "Convertir en JPG"
    Set-ItemProperty -Path $keyBase -Name "Icon" -Value "`"$exeDest`",0"

    $cmdKey = "$keyBase\command"
    New-Item -Path $cmdKey -Force | Out-Null
    Set-ItemProperty -Path $cmdKey -Name "(Default)" -Value "`"$exeDest`" `"%1`""
}

Write-Host ""
Write-Host "Installation terminee." -ForegroundColor Green
Write-Host "Clic droit sur un .heic pour utiliser." -ForegroundColor Cyan
Write-Host "(Si tu ne le vois pas, essaie 'Afficher plus d'options'.)" -ForegroundColor DarkGray
