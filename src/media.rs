use crate::gemini::ImageAttachment;
use base64::Engine;
use chrono::Local;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

pub fn load_image_from_file(path_str: &str) -> Result<ImageAttachment, String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err(format!("Arquivo de imagem não encontrado: {}", path_str));
    }

    let bytes = fs::read(path).map_err(|e| format!("Erro ao ler arquivo '{}': {}", path_str, e))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    let mime_type = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/jpeg",
    };

    let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "imagem.jpg".to_string());

    let data_base64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    Ok(ImageAttachment {
        filename,
        mime_type: mime_type.to_string(),
        data_base64,
    })
}

pub fn load_image_from_clipboard() -> Result<ImageAttachment, String> {
    let mut clipboard = arboard::Clipboard::new()
        .map_err(|e| format!("Falha ao acessar a área de transferência: {}", e))?;

    let img_data = clipboard
        .get_image()
        .map_err(|e| format!("Nenhuma imagem encontrada no clipboard: {}", e))?;

    let width = img_data.width as u32;
    let height = img_data.height as u32;
    let raw_bytes = img_data.bytes.into_owned();

    let img_buffer: image::ImageBuffer<image::Rgba<u8>, _> =
        image::ImageBuffer::from_raw(width, height, raw_bytes)
            .ok_or_else(|| "Buffer de imagem do clipboard inválido".to_string())?;

    let mut png_bytes = Vec::new();
    img_buffer
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| format!("Erro ao codificar imagem como PNG: {}", e))?;

    let filename = format!("clipboard_{}.png", Local::now().format("%Y%m%d_%H%M%S"));
    let data_base64 = base64::engine::general_purpose::STANDARD.encode(&png_bytes);

    Ok(ImageAttachment {
        filename,
        mime_type: "image/png".to_string(),
        data_base64,
    })
}

pub fn save_generated_image(
    vault_dir: &Path,
    prompt: &str,
    mime_type: &str,
    bytes: &[u8],
) -> Result<PathBuf, String> {
    let img_dir = vault_dir.join("01_inbox").join("imagens");
    fs::create_dir_all(&img_dir)
        .map_err(|e| format!("Erro ao criar diretório de imagens: {}", e))?;

    let ext = match mime_type {
        "image/png" => "png",
        "image/webp" => "webp",
        _ => "jpg",
    };

    let slug: String = prompt
        .chars()
        .take(30)
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let slug = slug.trim_matches('_');

    let filename = format!(
        "{}_{}.{}",
        Local::now().format("%Y%m%d_%H%M%S"),
        if slug.is_empty() { "gerada" } else { slug },
        ext
    );

    let target_path = img_dir.join(filename);
    fs::write(&target_path, bytes).map_err(|e| format!("Erro ao gravar imagem gerada: {}", e))?;

    Ok(target_path)
}

pub fn open_image_viewer(path: &Path) {
    let _ = std::process::Command::new("nsxiv").arg(path).spawn();
}
