# Heico - Installation du menu contextuel Windows
#
# Copie heico.exe dans %LOCALAPPDATA%\Heico\ et ajoute "Convertir en JPG"
# au menu contextuel des fichiers .heic / .HEIC pour l'utilisateur courant.
#
# Aucun droit administrateur requis.

$ErrorActionPreference = "Stop"

# Pause systematique a la fin (succes ou erreur) pour que la fenetre ne se
# ferme pas avant que l'utilisateur ait lu le resultat, notamment quand
# install.ps1 est lance via clic droit > Executer avec PowerShell.
trap {
    Write-Host ""
    Write-Host "Erreur : $_" -ForegroundColor Red
    Read-Host "Appuie sur Entree pour fermer"
    exit 1
}

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition

# Cherche heico.exe d'abord cote-a-cote (layout du zip de release),
# puis dans ..\target\release\ (layout de dev cargo build).
$candidateExePaths = @(
    (Join-Path $scriptDir "heico.exe"),
    (Join-Path $scriptDir "..\target\release\heico.exe")
)
$exeSource = $candidateExePaths | Where-Object { Test-Path $_ } | Select-Object -First 1

if (-not $exeSource) {
    Write-Host "heico.exe introuvable. Cherche aux emplacements :" -ForegroundColor Red
    foreach ($p in $candidateExePaths) { Write-Host "  - $p" -ForegroundColor DarkGray }
    Write-Host "Si tu as telecharge le zip de release, lance install.ps1 depuis le dossier dezippe (pas un raccourci ailleurs)." -ForegroundColor Yellow
    Read-Host "Appuie sur Entree pour fermer"
    exit 1
}

$installDir = Join-Path $env:LOCALAPPDATA "Heico"
$exeDest = Join-Path $installDir "heico.exe"

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

Copy-Item $exeSource $exeDest -Force
Write-Host "Copie : $exeDest" -ForegroundColor Green

# Copie les DLL natives (heif.dll, libde265.dll, libx265.dll) depuis vcpkg ou a cote du binaire.
$dllNames = @("heif.dll", "libde265.dll", "libx265.dll", "dav1d.dll")

$dllCandidates = @()
foreach ($name in $dllNames) {
    $dllCandidates += (Join-Path (Split-Path -Parent $exeSource) $name)
}

if ($env:VCPKG_ROOT) {
    $vcpkgBin = Join-Path $env:VCPKG_ROOT "installed\x64-windows\bin"
    if (Test-Path $vcpkgBin) {
        foreach ($name in $dllNames) {
            $dllCandidates += (Join-Path $vcpkgBin $name)
        }
    }
}

$copiedDlls = @{}
foreach ($dll in $dllCandidates) {
    if ((Test-Path $dll) -and -not $copiedDlls.ContainsKey((Split-Path -Leaf $dll))) {
        Copy-Item $dll (Join-Path $installDir (Split-Path -Leaf $dll)) -Force
        Write-Host "  + $(Split-Path -Leaf $dll)" -ForegroundColor DarkGray
        $copiedDlls[(Split-Path -Leaf $dll)] = $true
    }
}

# Cle registre pour .heic / .heif / .png (Windows traite parfois les casses separement)
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
    New-Item -Path $keyBase -Force | Out-Null
    Set-ItemProperty -Path $keyBase -Name "(Default)" -Value "Convertir en JPG"
    Set-ItemProperty -Path $keyBase -Name "Icon" -Value "`"$exeDest`",0"
    # MultiSelectModel=Player : appelle heico une seule fois avec tous les
    # fichiers en arguments, au lieu d'un processus par fichier. Sans ca,
    # Windows masque l'entree au-dela de ~15 fichiers selectionnes.
    Set-ItemProperty -Path $keyBase -Name "MultiSelectModel" -Value "Player"

    $cmdKey = "$keyBase\command"
    New-Item -Path $cmdKey -Force | Out-Null
    Set-ItemProperty -Path $cmdKey -Name "(Default)" -Value "`"$exeDest`" `"%1`""
}

Write-Host ""
Write-Host "Installation terminee." -ForegroundColor Green
Write-Host "Clic droit sur un .heic / .heif / .png / .webp / .tif / .bmp / .gif / .avif pour utiliser." -ForegroundColor Cyan
Write-Host "(Si tu ne le vois pas, essaie 'Afficher plus d'options'.)" -ForegroundColor DarkGray
Write-Host ""
Read-Host "Appuie sur Entree pour fermer"
