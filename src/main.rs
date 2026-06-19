// En release on cible le subsystem "windows" pour qu'aucune fenetre console
// ne flashe quand l'exe est appele depuis le menu contextuel de l'explorateur.
// En debug on garde le subsystem "console" pour pouvoir voir les prints au dev.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

mod notify;

// true = console parent attachee (mode CLI, eprintln visible), false = lance
// depuis l'explorateur (subsystem windows, eprintln dans le vide -> toast).
#[cfg(windows)]
fn attach_parent_console() -> bool {
    use windows_sys::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) != 0 }
}

#[cfg(not(windows))]
fn attach_parent_console() -> bool {
    true
}

/// Convertit un ou plusieurs fichiers HEIC en JPG.
#[derive(Parser, Debug)]
#[command(name = "heico", version, about, long_about = None)]
struct Cli {
    /// Fichiers HEIC à convertir (un ou plusieurs).
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Qualité JPG (1-100).
    #[arg(short, long, default_value_t = 88)]
    quality: u8,

    /// Dossier de sortie. Par défaut, même dossier que le fichier source.
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Écrase le fichier JPG s'il existe déjà.
    #[arg(short, long)]
    force: bool,

    /// Ne préserve pas les métadonnées EXIF.
    #[arg(long)]
    no_exif: bool,
}

fn main() {
    let console_attached = attach_parent_console();
    let cli = Cli::parse();

    if cli.quality == 0 || cli.quality > 100 {
        eprintln!("Erreur : la qualité doit être entre 1 et 100.");
        std::process::exit(2);
    }

    if let Some(ref out) = cli.output {
        if !out.exists() {
            std::fs::create_dir_all(out).unwrap_or_else(|e| {
                eprintln!("Erreur : impossible de créer le dossier de sortie : {e}");
                std::process::exit(2);
            });
        }
    }

    let total = cli.files.len();
    let pb = if total > 1 {
        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::with_template("{bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        Some(pb)
    } else {
        None
    };

    let success = AtomicUsize::new(0);
    let skipped = AtomicUsize::new(0);
    let failed = AtomicUsize::new(0);
    // Premier message d'erreur capture pour le toast en mode explorateur. On garde
    // aussi le nom du fichier source associe pour le contexte utilisateur.
    let first_err: Mutex<Option<String>> = Mutex::new(None);

    cli.files.par_iter().for_each(|src| {
        let result = convert_one(
            src,
            cli.output.as_deref(),
            cli.quality,
            cli.force,
            !cli.no_exif,
        );
        match result {
            Ok(ConvertOutcome::Converted(dst)) => {
                success.fetch_add(1, Ordering::Relaxed);
                if let Some(ref pb) = pb {
                    pb.set_message(format!("OK {}", dst.display()));
                    pb.inc(1);
                } else {
                    println!("OK : {}", dst.display());
                }
            }
            Ok(ConvertOutcome::Skipped(dst)) => {
                skipped.fetch_add(1, Ordering::Relaxed);
                if let Some(ref pb) = pb {
                    pb.set_message(format!("SKIP {}", dst.display()));
                    pb.inc(1);
                } else {
                    eprintln!(
                        "Ignoré (existe déjà) : {}. Utilise --force pour écraser.",
                        dst.display()
                    );
                }
            }
            Err(e) => {
                failed.fetch_add(1, Ordering::Relaxed);
                if let Some(ref pb) = pb {
                    pb.set_message(format!("ERR {}", src.display()));
                    pb.inc(1);
                }
                let fname = src.file_name().and_then(|n| n.to_str()).unwrap_or("?");
                let msg = format!("{fname} : {e:#}");
                eprintln!("Erreur sur {} : {e:#}", src.display());
                let mut slot = first_err.lock().unwrap();
                if slot.is_none() {
                    *slot = Some(msg);
                }
            }
        }
    });

    if let Some(pb) = pb {
        pb.finish_with_message("terminé");
    }

    let ok = success.load(Ordering::Relaxed);
    let sk = skipped.load(Ordering::Relaxed);
    let er = failed.load(Ordering::Relaxed);
    if total > 1 {
        println!("\n{ok} converti(s), {sk} ignoré(s), {er} en erreur.");
    }

    if er > 0 {
        // En mode explorateur (subsystem windows, pas de console parent), eprintln!
        // part dans le vide. Un toast Windows est la seule chance pour l'utilisateur
        // de voir qu'un fichier a echoue. En mode CLI, on s'abstient (eprintln suffit).
        if !console_attached {
            let first = first_err.lock().unwrap().take();
            notify::show_error_summary(er, total, first.as_deref());
        }
        std::process::exit(1);
    }
}

enum ConvertOutcome {
    Converted(PathBuf),
    Skipped(PathBuf),
}

fn convert_one(
    src: &Path,
    output_dir: Option<&Path>,
    quality: u8,
    force: bool,
    keep_exif: bool,
) -> Result<ConvertOutcome> {
    if !src.exists() {
        return Err(anyhow!("fichier introuvable"));
    }

    let dst = compute_destination(src, output_dir)?;

    if dst.exists() && !force {
        return Ok(ConvertOutcome::Skipped(dst));
    }

    let ext = src
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase());
    let decoded = match ext.as_deref() {
        Some("heic") | Some("heif") => decode_heic(src, quality)?,
        Some("png") | Some("webp") | Some("tif") | Some("tiff") | Some("bmp") | Some("gif")
        | Some("avif") => decode_with_image_crate(src, quality)?,
        Some(other) => return Err(anyhow!("format non supporté : .{other}")),
        None => return Err(anyhow!("extension manquante")),
    };

    let exif = if keep_exif {
        decoded.exif.map(|mut e| {
            // libheif a deja applique la rotation EXIF sur les pixels au decodage,
            // donc on neutralise le tag Orientation pour eviter qu'un viewer JPG
            // ne rotate une seconde fois.
            neutralize_exif_orientation(&mut e);
            e
        })
    } else {
        None
    };

    let final_bytes = finalize_jpeg(
        &decoded.jpeg_bytes,
        decoded.icc_profile.as_deref(),
        exif.as_deref(),
    )
    .unwrap_or(decoded.jpeg_bytes);

    std::fs::write(&dst, final_bytes).with_context(|| format!("écriture de {}", dst.display()))?;

    Ok(ConvertOutcome::Converted(dst))
}

struct DecodedSrc {
    jpeg_bytes: Vec<u8>,
    icc_profile: Option<Vec<u8>>,
    exif: Option<Vec<u8>>,
}

fn compute_destination(src: &Path, output_dir: Option<&Path>) -> Result<PathBuf> {
    let stem = src
        .file_stem()
        .ok_or_else(|| anyhow!("nom de fichier invalide"))?;
    let mut dst = match output_dir {
        Some(dir) => dir.to_path_buf(),
        None => src
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from(".")),
    };
    dst.push(stem);
    dst.set_extension("jpg");
    Ok(dst)
}

fn decode_heic(src: &Path, quality: u8) -> Result<DecodedSrc> {
    use image::{codecs::jpeg::JpegEncoder, ColorType};
    use libheif_rs::{ColorSpace, HeifContext, ItemId, LibHeif, RgbChroma};

    let lib = LibHeif::new();
    let ctx = HeifContext::read_from_file(
        src.to_str()
            .ok_or_else(|| anyhow!("chemin non-UTF8 : {}", src.display()))?,
    )
    .with_context(|| "ouverture HEIC")?;
    let handle = ctx
        .primary_image_handle()
        .with_context(|| "image primaire")?;

    // Profil ICC (Display P3 sur iPhone) : extrait depuis le handle source pour le
    // reinjecter tel quel dans le JPG. Sans ca, les viewers JPG supposent sRGB et
    // les couleurs paraissent ternes sur les photos large gamut.
    let icc_profile = handle.color_profile_raw().map(|p| p.data);

    // EXIF : le payload libheif est prefixe par un offset (4 octets BE) au TIFF header.
    let exif = {
        let count = handle.number_of_metadata_blocks(b"Exif");
        if count > 0 {
            let mut ids: Vec<ItemId> = vec![0; count as usize];
            handle.metadata_block_ids(&mut ids, b"Exif");
            ids.first()
                .and_then(|&id| handle.metadata(id).ok())
                .and_then(|raw| {
                    if raw.len() < 4 {
                        return None;
                    }
                    let offset = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize;
                    let start = 4 + offset;
                    if start >= raw.len() {
                        return None;
                    }
                    Some(raw[start..].to_vec())
                })
        } else {
            None
        }
    };

    let img = lib
        .decode(&handle, ColorSpace::Rgb(RgbChroma::Rgb), None)
        .with_context(|| "décodage HEIC")?;

    let planes = img.planes();
    let plane = planes
        .interleaved
        .ok_or_else(|| anyhow!("plan RGB indisponible"))?;

    let width = plane.width;
    let height = plane.height;
    let stride = plane.stride;
    let data = plane.data;

    // Recompact en (width*3) si stride > width*3.
    let row_bytes = (width as usize) * 3;
    let packed: Vec<u8> = if stride == row_bytes {
        data.to_vec()
    } else {
        let mut buf = Vec::with_capacity(row_bytes * height as usize);
        for y in 0..height as usize {
            let start = y * stride;
            buf.extend_from_slice(&data[start..start + row_bytes]);
        }
        buf
    };

    let mut jpeg_bytes = Vec::with_capacity(row_bytes * height as usize / 4);
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, quality);
    encoder
        .encode(&packed, width, height, ColorType::Rgb8.into())
        .with_context(|| "encodage JPEG")?;

    Ok(DecodedSrc {
        jpeg_bytes,
        icc_profile,
        exif,
    })
}

fn decode_with_image_crate(src: &Path, quality: u8) -> Result<DecodedSrc> {
    use image::{DynamicImage, ImageDecoder, ImageReader};

    let reader = ImageReader::open(src)
        .with_context(|| "ouverture image")?
        .with_guessed_format()
        .with_context(|| "détection format")?;
    let mut decoder = reader.into_decoder().with_context(|| "init décodeur")?;
    let icc_profile = decoder.icc_profile().ok().flatten();
    let img = DynamicImage::from_decoder(decoder).with_context(|| "lecture pixels")?;
    finalize_decoded_image(img, quality, icc_profile)
}

fn finalize_decoded_image(
    img: image::DynamicImage,
    quality: u8,
    icc_profile: Option<Vec<u8>>,
) -> Result<DecodedSrc> {
    use image::codecs::jpeg::JpegEncoder;
    use image::{ColorType, DynamicImage};

    let (width, height) = (img.width(), img.height());

    // JPG ne supporte pas la transparence : composite RGBA / LumaA sur fond blanc.
    let rgb_bytes: Vec<u8> = match img {
        DynamicImage::ImageRgb8(buf) => buf.into_raw(),
        DynamicImage::ImageRgba8(buf) => composite_rgba_on_white(buf.as_raw(), width, height),
        DynamicImage::ImageLuma8(buf) => {
            let mut out = Vec::with_capacity(buf.len() * 3);
            for px in buf.iter() {
                out.extend_from_slice(&[*px, *px, *px]);
            }
            out
        }
        DynamicImage::ImageLumaA8(buf) => {
            let raw = buf.into_raw();
            let mut out = Vec::with_capacity((raw.len() / 2) * 3);
            for chunk in raw.chunks_exact(2) {
                let (l, a) = (chunk[0] as u16, chunk[1] as u16);
                let inv = 255 - a;
                let v = ((l * a + 255 * inv) / 255) as u8;
                out.extend_from_slice(&[v, v, v]);
            }
            out
        }
        other => other.to_rgb8().into_raw(),
    };

    let mut jpeg_bytes = Vec::with_capacity(rgb_bytes.len() / 4);
    let mut encoder = JpegEncoder::new_with_quality(&mut jpeg_bytes, quality);
    encoder
        .encode(&rgb_bytes, width, height, ColorType::Rgb8.into())
        .with_context(|| "encodage JPEG")?;

    Ok(DecodedSrc {
        jpeg_bytes,
        icc_profile,
        exif: None,
    })
}

fn composite_rgba_on_white(rgba: &[u8], width: u32, height: u32) -> Vec<u8> {
    let pixels = (width as usize) * (height as usize);
    let mut out = Vec::with_capacity(pixels * 3);
    for chunk in rgba.chunks_exact(4) {
        let (r, g, b, a) = (
            chunk[0] as u16,
            chunk[1] as u16,
            chunk[2] as u16,
            chunk[3] as u16,
        );
        let inv = 255 - a;
        out.push(((r * a + 255 * inv) / 255) as u8);
        out.push(((g * a + 255 * inv) / 255) as u8);
        out.push(((b * a + 255 * inv) / 255) as u8);
    }
    out
}

/// Parcourt l'IFD0 du blob EXIF (format TIFF) et remet le tag Orientation (0x0112)
/// a la valeur 1 (Normal). Operation no-op si le blob n'est pas un TIFF valide ou
/// si le tag est absent.
fn neutralize_exif_orientation(exif: &mut [u8]) {
    if exif.len() < 8 {
        return;
    }
    let little_endian = match &exif[0..2] {
        b"II" => true,
        b"MM" => false,
        _ => return,
    };
    let read_u16 = |buf: &[u8], off: usize| -> Option<u16> {
        if off + 2 > buf.len() {
            return None;
        }
        Some(if little_endian {
            u16::from_le_bytes([buf[off], buf[off + 1]])
        } else {
            u16::from_be_bytes([buf[off], buf[off + 1]])
        })
    };
    let read_u32 = |buf: &[u8], off: usize| -> Option<u32> {
        if off + 4 > buf.len() {
            return None;
        }
        Some(if little_endian {
            u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
        } else {
            u32::from_be_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
        })
    };

    if read_u16(exif, 2) != Some(42) {
        return;
    }
    let Some(ifd0_offset) = read_u32(exif, 4) else {
        return;
    };
    let ifd0_offset = ifd0_offset as usize;
    let Some(entry_count) = read_u16(exif, ifd0_offset) else {
        return;
    };
    let entries_start = ifd0_offset + 2;

    for i in 0..(entry_count as usize) {
        let entry_off = entries_start + i * 12;
        if entry_off + 12 > exif.len() {
            break;
        }
        let Some(tag) = read_u16(exif, entry_off) else {
            break;
        };
        if tag == 0x0112 {
            // Tag Orientation. Type SHORT (3), count 1, valeur inline a entry_off+8.
            if little_endian {
                exif[entry_off + 8] = 1;
                exif[entry_off + 9] = 0;
            } else {
                exif[entry_off + 8] = 0;
                exif[entry_off + 9] = 1;
            }
            return;
        }
    }
}

fn finalize_jpeg(jpeg: &[u8], icc: Option<&[u8]>, exif: Option<&[u8]>) -> Result<Vec<u8>> {
    use img_parts::jpeg::Jpeg;
    use img_parts::{ImageEXIF, ImageICC};

    let mut img = Jpeg::from_bytes(jpeg.to_vec().into())?;
    if let Some(i) = icc {
        img.set_icc_profile(Some(i.to_vec().into()));
    }
    if let Some(e) = exif {
        img.set_exif(Some(e.to_vec().into()));
    }
    let cap =
        jpeg.len() + icc.map(|v| v.len()).unwrap_or(0) + exif.map(|v| v.len()).unwrap_or(0) + 64;
    let mut out = Vec::with_capacity(cap);
    img.encoder().write_to(&mut out)?;
    Ok(out)
}
