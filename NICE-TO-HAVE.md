# Nice to have

Pistes d'évolution V2+ pour Heico. Aucune n'est priorisée : à confronter au moment opportun avec le critère filtre **"est-ce que ça casse la promesse simplicité ?"** (un clic, fidèle, sans friction, sans config, sans réseau).

## Candidats forts

### 1. Variante "Convertir et supprimer l'original"

Nouvelle entrée registre `HeicoConvertAndDelete` à côté de l'actuelle. Cas d'usage massif : import iPhone, on garde uniquement le JPG. Demande un dialog de confirmation Windows si plus d'un fichier (avec checkbox "ne plus demander" stockée dans `%LOCALAPPDATA%\Heico\config.json`).

Effort : faible. Toucher `main.rs` (option CLI `--delete-source`) + `install.ps1` (verbe registre supplémentaire).

### 2. Sous-menu de redimensionnement

Trois verbes registre côte à côte :
- "Convertir en JPG" (taille originale, actuel)
- "Convertir en JPG (max 2048px)" pour mail/social
- "Convertir en JPG (max 1024px)" pour preview web

Chaque action reste un seul clic, donc la promesse tient. Implémentation : flag CLI `--max <px>` + 2 verbes registre. Resize avec préservation du ratio, utiliser `image::imageops::resize` avec filtre `Lanczos3`.

Effort : faible à moyen.

### 3. Publication sur winget

`winget install bastienbrasseur.heico`. Effort moyen : créer un manifest YAML + PR sur `microsoft/winget-pkgs`. Validation auto par leur CI, merge sous quelques jours en général. Adoption x10 chez les users tech qui scriptent leur setup PC.

Prérequis : avoir une première release stable signée (ou au moins reproductible).

### 4. Toast Windows en cas d'erreur

Aujourd'hui, en mode clic droit (subsystem `windows`), `eprintln!` part dans le vide : l'utilisateur ne voit pas qu'un fichier a échoué. Fix : afficher une toast notification native sur les erreurs critiques (fichier illisible, disque plein, format inconnu).

Implémentation : crate `winrt-notification` ou `windows::UI::Notifications`. Toast unique par batch ("3 fichiers en erreur sur 50, voir le dossier"), pas une par fichier.

Effort : faible.

## Intéressants mais à débattre

### 5. Check de mise à jour opt-in

Commande CLI `heico --check-update` qui interroge GitHub Releases. **Ne pas** faire de check auto au démarrage : ça casserait "100% offline, aucun service tiers".

Effort : faible.

### 6. Format de sortie alternatif (WebP)

Flag CLI `--format webp` pour les users avancés. **Ne pas** ajouter d'entrée menu contextuel "Convertir en WebP", ça diluerait l'identité du tool ("vers JPG"). Reste en CLI uniquement.

Effort : faible (le crate `image` sait encoder WebP).

## À éviter (casse la promesse)

- GUI de paramétrage. Dialog avec preview, qualité ajustable, etc. Si tu veux ça, t'utilises XnConvert ou IrfanView.
- Watch folder / convert-on-arrival. Casse "zéro background, zéro service".
- Profils de conversion configurables. Multiplie les chemins, complique le mental model.
- Telemetry "anonyme" pour suivre l'usage. Non, jamais.
- Plugins / extensions tierces. Pas d'ambition de devenir une plateforme.

## Décisions techniques en suspens

- **Resize : quand ?** Si on garde le ratio source mais qu'une seule dimension dépasse `--max`, on resize uniquement celle qui dépasse, ratio préservé. Cas pathologique : image carrée 4096x4096 avec `--max 2048` → 2048x2048, OK. Image 5000x500 avec `--max 2048` → 2048x205, OK aussi.
- **Toast d'erreur multi-fichiers** : agréger en une seule notification ou une par fichier ? Une seule (sinon spam visuel).
- **Suppression de l'original** : poubelle Windows ou suppression sèche ? Poubelle, plus safe (l'utilisateur peut restaurer si erreur de conversion silencieuse).
