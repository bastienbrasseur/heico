# Heico

Convertisseur HEIC / PNG vers JPG pour Windows. Clic droit sur le fichier, **Convertir en JPG**, voilà.

Pensé pour deux cas concrets :
- Les HEIC d'iPhone qui ne s'ouvrent nulle part sans payer une extension à Microsoft.
- Les gros PNG full-détail générés par IA qu'on veut en JPG pour gagner 5x en poids.

Features :
- Conversion par clic droit dans l'explorateur Windows
- Batch parallèle si tu sélectionnes plusieurs fichiers
- HEIC : préserve l'EXIF (date, orientation, géoloc) et le profil ICC (Display P3)
- PNG : préserve l'ICC, composite transparence sur fond blanc
- 100% offline, aucun service tiers, aucune télémétrie
- Open source MIT

## Installation

À venir : un installeur prêt à l'emploi en release GitHub.

En attendant, build from source (section ci-dessous).

## Utilisation

### Depuis l'explorateur Windows

Clic droit sur un `.heic`, `.heif` ou `.png` → **Convertir en JPG** → un `.jpg` apparaît à côté.

### Depuis la ligne de commande

```powershell
# Un fichier
heico photo.heic

# Plusieurs
heico *.heic

# Avec options
heico -q 90 -o C:\Sortie\ --force photo.heic
```

Options :

| Option | Défaut | Description |
|---|---|---|
| `-q, --quality <1-100>` | `92` | Qualité JPG |
| `-o, --output <dir>` | dossier source | Dossier de sortie |
| `-f, --force` | off | Écrase le JPG s'il existe |
| `--no-exif` | off | N'embarque pas l'EXIF |

## Build from source

### Prérequis

- [Rust stable](https://rustup.rs/) (1.75+)
- [vcpkg](https://github.com/microsoft/vcpkg) pour `libheif`

```powershell
# vcpkg (une fois)
git clone https://github.com/microsoft/vcpkg C:\vcpkg
C:\vcpkg\bootstrap-vcpkg.bat

# libheif (x64)
C:\vcpkg\vcpkg install libheif:x64-windows-static
```

### Build

```powershell
$env:VCPKGRS_TRIPLET = "x64-windows-static"
$env:VCPKG_ROOT = "C:\vcpkg"
cargo build --release
```

Le binaire est dans `target\release\heico.exe`.

## Désinstallation

```powershell
.\installer\uninstall.ps1
```

Retire la clé registre du menu contextuel et supprime `%LOCALAPPDATA%\Heico\`.

## Licence

MIT, voir [LICENSE](LICENSE).
