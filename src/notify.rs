// Toast Windows pour signaler les erreurs en mode "lance depuis l'explorateur",
// ou eprintln! part dans le vide (subsystem windows, pas de console).
// Un seul toast aggrege par batch, jamais un par fichier.

#[cfg(windows)]
pub fn show_error_summary(failed: usize, total: usize, first_err: Option<&str>) {
    use tauri_winrt_notification::{Duration, Sound, Toast};

    // AUMID enregistre par l'installer Inno Setup dans HKCU. En l'absence de cle
    // (build dev sans installer), tauri-winrt-notification fallback sur l'AUMID
    // PowerShell : le toast s'affiche quand meme mais avec une icone generique.
    const AUMID: &str = "Apptic.Heico";

    let title = "Heico - conversion incomplète";
    let body = match (failed, total, first_err) {
        (1, 1, Some(msg)) => format!("Echec : {msg}"),
        (1, _, Some(msg)) => format!("1 fichier en erreur sur {total}. {msg}"),
        (n, _, _) => format!("{n} fichiers en erreur sur {total}."),
    };

    let _ = Toast::new(AUMID)
        .title(title)
        .text1(&body)
        .sound(Some(Sound::Default))
        .duration(Duration::Short)
        .show();
}

#[cfg(not(windows))]
pub fn show_error_summary(_failed: usize, _total: usize, _first_err: Option<&str>) {}
