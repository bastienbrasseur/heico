use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use anyhow::{anyhow, Context, Result};
use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

/// Convertit un ou plusieurs fichiers HEIC en JPG.
#[derive(Parser, Debug)]
#[command(name = "heico", version, about, long_about = None)]
struct Cli {
    /// Fichiers HEIC à convertir (un ou plusieurs).
    #[arg(required = true)]
    files: Vec<PathBuf>,

    /// Qualité JPG (1-100).
    #[arg(short, long, default_value_t = 92)]
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
                eprintln!("Erreur sur {} : {e:#}", src.display());
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

    let jpg_bytes = decode_heic_to_jpeg(src, quality)?;

    let final_bytes = if keep_exif {
        match extract_exif(src) {
            Ok(Some(exif)) => inject_exif_into_jpeg(&jpg_bytes, &exif).unwrap_or(jpg_bytes),
            _ => jpg_bytes,
        }
    } else {
        jpg_bytes
    };

    std::fs::write(&dst, final_bytes).with_context(|| format!("écriture de {}", dst.display()))?;

    Ok(ConvertOutcome::Converted(dst))
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

fn decode_heic_to_jpeg(src: &Path, quality: u8) -> Result<Vec<u8>> {
    use image::{codecs::jpeg::JpegEncoder, ColorType};
    use libheif_rs::{ColorSpace, HeifContext, LibHeif, RgbChroma};

    let lib = LibHeif::new();
    let ctx = HeifContext::read_from_file(
        src.to_str()
            .ok_or_else(|| anyhow!("chemin non-UTF8 : {}", src.display()))?,
    )
    .with_context(|| "ouverture HEIC")?;
    let handle = ctx
        .primary_image_handle()
        .with_context(|| "image primaire")?;

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

    let mut out = Vec::with_capacity(row_bytes * height as usize / 4);
    let mut encoder = JpegEncoder::new_with_quality(&mut out, quality);
    encoder
        .encode(&packed, width, height, ColorType::Rgb8.into())
        .with_context(|| "encodage JPEG")?;

    Ok(out)
}

fn extract_exif(src: &Path) -> Result<Option<Vec<u8>>> {
    use libheif_rs::{HeifContext, ItemId};

    let ctx = HeifContext::read_from_file(src.to_str().ok_or_else(|| anyhow!("chemin non-UTF8"))?)?;
    let handle = ctx.primary_image_handle()?;

    let exif_tag = b"Exif";
    let count = handle.number_of_metadata_blocks(exif_tag);
    if count <= 0 {
        return Ok(None);
    }
    let mut ids: Vec<ItemId> = vec![0; count as usize];
    handle.metadata_block_ids(&mut ids, exif_tag);
    let Some(&id) = ids.first() else {
        return Ok(None);
    };
    let raw = handle.metadata(id)?;
    // libheif préfixe l'EXIF par 4 octets d'offset au TIFF header.
    if raw.len() < 4 {
        return Ok(None);
    }
    let offset = u32::from_be_bytes([raw[0], raw[1], raw[2], raw[3]]) as usize;
    let start = 4 + offset;
    if start >= raw.len() {
        return Ok(None);
    }
    Ok(Some(raw[start..].to_vec()))
}

fn inject_exif_into_jpeg(jpeg: &[u8], exif: &[u8]) -> Result<Vec<u8>> {
    use img_parts::jpeg::Jpeg;
    use img_parts::ImageEXIF;

    let mut img = Jpeg::from_bytes(jpeg.to_vec().into())?;
    img.set_exif(Some(exif.to_vec().into()));
    let mut out = Vec::with_capacity(jpeg.len() + exif.len() + 32);
    img.encoder().write_to(&mut out)?;
    Ok(out)
}
