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

# Copie les DLL natives (heif.dll, libde265.dll, libx265.dll) depuis vcpkg ou a cote du binaire.
$dllCandidates = @(
    (Join-Path (Split-Path -Parent $exeSource) "heif.dll"),
    (Join-Path (Split-Path -Parent $exeSource) "libde265.dll"),
    (Join-Path (Split-Path -Parent $exeSource) "libx265.dll")
)

if ($env:VCPKG_ROOT) {
    $vcpkgBin = Join-Path $env:VCPKG_ROOT "installed\x64-windows\bin"
    if (Test-Path $vcpkgBin) {
        $dllCandidates += @(
            (Join-Path $vcpkgBin "heif.dll"),
            (Join-Path $vcpkgBin "libde265.dll"),
            (Join-Path $vcpkgBin "libx265.dll")
        )
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
