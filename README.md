# Heico

[![CI](https://github.com/bastienbrasseur/heico/actions/workflows/ci.yml/badge.svg)](https://github.com/bastienbrasseur/heico/actions/workflows/ci.yml)

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

## Pourquoi pas une autre solution ?

- **L'extension HEIF de Microsoft** est payante (0,99 €) et n'ouvre que le HEIC dans la visionneuse Windows. Elle ne convertit rien.
- **Les convertisseurs en ligne** uploadent tes photos sur leurs serveurs. Pas idéal pour les photos perso.
- **IrfanView, XnConvert, GIMP** font le job mais demandent d'ouvrir une appli, charger l'image, choisir un format, exporter. Trop d'étapes pour la tâche "j'ai 30 HEIC à passer en JPG".

Heico est une seule action : clic droit, **Convertir en JPG**. Rien à apprendre, rien à configurer.

## Installation

1. Télécharge le dernier zip sur la page [Releases](https://github.com/bastienbrasseur/heico/releases).
2. Dézippe où tu veux (par exemple `C:\Tools\heico\`).
3. Lance `install.ps1` une fois (clic droit → **Exécuter avec PowerShell**, aucun droit admin requis).

L'installeur copie `heico.exe` + les DLL natives dans `%LOCALAPPDATA%\Heico\` et ajoute l'entrée **Convertir en JPG** au menu contextuel pour l'utilisateur courant.

Pour build toi-même depuis les sources, voir plus bas.

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
| `-q, --quality <1-100>` | `88` | Qualité JPG |
| `-o, --output <dir>` | dossier source | Dossier de sortie |
| `-f, --force` | off | Écrase le JPG s'il existe |
| `--no-exif` | off | N'embarque pas l'EXIF |

## Limitations

- **Windows x64 uniquement.** Pas de build macOS/Linux prévu (l'intérêt du projet est l'intégration menu contextuel Windows).
- **Pas d'interface graphique.** L'outil vit dans l'explorateur et la ligne de commande, c'est volontaire.
- **HEIC multi-images (Burst, Live Photos)** : seule l'image primaire est convertie, les autres frames sont ignorées.
- **AVIF animés** : seule la première frame est extraite.
- **GIF animés** : seule la première frame est convertie. Pour un GIF animé vers MP4, utilise un autre outil.
- **Pas de signature de code.** Le binaire n'est pas signé, donc Windows SmartScreen peut afficher un avertissement au premier lancement (voir Dépannage).

## Performance

Sur un laptop standard (CPU 8 cœurs, SSD NVMe), conversion d'un dossier de 100 HEIC iPhone (3 à 4 MB chacun) en parallèle : **environ 12 secondes**, soit ~8 photos/seconde. Le facteur limitant est le décodage HEIC (libheif), pas l'encodage JPEG ni le disque.

## Dépannage

**Le menu contextuel "Convertir en JPG" n'apparaît pas.**
Sur Windows 11, le clic droit affiche d'abord un menu réduit. Clique sur **Afficher plus d'options** (ou fais `Shift + clic droit`) pour voir l'entrée Heico. Le menu Windows 11 "moderne" ne supporte que les entrées packagées MSIX, pas les clés registre classiques.

**Erreur "DLL manquante : heif.dll" (ou libde265 / dav1d / libx265).**
Relance `install.ps1` depuis le zip téléchargé. Si tu as build toi-même, vérifie que les DLL vcpkg ont bien été copiées dans `%LOCALAPPDATA%\Heico\`.

**Windows SmartScreen bloque l'exe au premier lancement.**
Le binaire n'est pas signé (signature de code = ~300 €/an, pas justifié pour un outil gratuit). Clique sur **Informations complémentaires** puis **Exécuter quand même**. Tu peux aussi inspecter le code source de ce repo et build toi-même si tu préfères.

**La photo convertie a les couleurs ternes.**
Vérifie que ton viewer JPG sait lire les profils ICC embarqués. Photos (Windows) et la plupart des navigateurs le font. Certaines vieilles visionneuses ignorent l'ICC et affichent le JPG en sRGB générique.

**Le batch n'est pas plus rapide qu'un fichier à la fois.**
Heico parallélise via `rayon` (un thread par cœur). Si tu lances la conversion depuis l'explorateur sur 50 fichiers, ils sont traités en parallèle. Si tu lances 50 fois `heico fichier.heic` à la main, chaque processus utilise un seul thread.

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
