# Heico

Convertisseur image vers JPG pour Windows. Clic droit sur le fichier, **Convertir en JPG**, voilà.

Formats supportés en entrée : **HEIC, HEIF, PNG, WebP, TIFF, BMP, GIF, AVIF**.

Pensé pour les cas concrets du quotidien :
- Les HEIC d'iPhone qui ne s'ouvrent nulle part sans payer une extension à Microsoft.
- Les gros PNG full-détail générés par IA qu'on veut en JPG pour gagner 5x en poids.
- Les WebP téléchargés du web que personne ne sait ouvrir.
- Tout autre format image traînant dans tes téléchargements.

Features :
- Conversion par clic droit dans l'explorateur Windows
- Batch parallèle si tu sélectionnes plusieurs fichiers
- Préserve le profil ICC (couleurs fidèles, notamment Display P3 iPhone)
- HEIC : préserve aussi l'EXIF (date, orientation, géoloc)
- Transparence : composite automatique sur fond blanc
- 100% offline, aucun service tiers, aucune télémétrie
- Open source MIT

## Installation

À venir : un installeur prêt à l'emploi en release GitHub.

En attendant, build from source (section ci-dessous).

## Utilisation

### Depuis l'explorateur Windows

Clic droit sur un fichier `.heic`, `.heif`, `.png`, `.webp`, `.tif`, `.bmp`, `.gif` ou `.avif` → **Convertir en JPG** → un `.jpg` apparaît à côté.

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

- [Rust stable](https://rustup.rs/) (1.79+)
- [vcpkg](https://github.com/microsoft/vcpkg) pour `libheif` et `dav1d`
- `pkg-config` (via scoop : `scoop install pkg-config`)

```powershell
# vcpkg (une fois)
git clone https://github.com/microsoft/vcpkg C:\vcpkg
C:\vcpkg\bootstrap-vcpkg.bat

# Libs natives (x64 dynamiques)
C:\vcpkg\vcpkg install libheif:x64-windows dav1d:x64-windows
```

### Build

```powershell
$env:VCPKG_ROOT = "C:\vcpkg"
$env:PKG_CONFIG_PATH = "C:\vcpkg\installed\x64-windows\lib\pkgconfig"
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
