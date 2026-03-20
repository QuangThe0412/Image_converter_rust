use image::{DynamicImage, ImageOutputFormat, ImageBuffer, Rgba, Rgb};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write, Cursor};
use std::path::Path;
use base64::{engine::general_purpose, Engine as _};
use anyhow::Result;

const DDJ_HEADER: [u8; 20] = [
    0x4A, 0x4D, 0x58, 0x56, 0x44, 0x44, 0x4A, 0x20, 0x31, 0x30, 0x30, 0x30, 
    0x88, 0x80, 0x00, 0x00, 0x03, 0x00, 0x00, 0x00
];

#[derive(Serialize, Deserialize)]
pub struct ImageMetadata {
    pub name: String,
    pub extension: String,
    pub width: u32,
    pub height: u32,
    pub preview: String, // base64
}

#[derive(Serialize, Deserialize)]
pub struct ConversionResult {
    pub success: bool,
    pub filename: String,
    pub error: Option<String>,
}

fn read_dds_content(path: &str) -> anyhow::Result<DynamicImage> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    let mut start_offset = 0;
    if buffer.len() >= 8 && &buffer[0..8] == b"JMXVDDJ " { // Check for DDJ magic number
        start_offset = 20;
    }

    if buffer.len() < start_offset + 128 { // Minimum DDS header size is 128 bytes
        return Err(anyhow::anyhow!("File too small for DDS"));
    }

    let dds_data = &buffer[start_offset..];
    let mut cursor = std::io::Cursor::new(dds_data);
    let dds = ddsfile::Dds::read(&mut cursor).map_err(|e| anyhow::anyhow!("DDS read error: {}", e))?;

    // Try image_dds first (good for compressed DXT)
    match image_dds::image_from_dds(&dds, 0) {
        Ok(img) => Ok(DynamicImage::ImageRgba8(img)),
        Err(_) => {
            // Fallback: manually handle some uncompressed formats using ddsfile
            let width = dds.get_width();
            let height = dds.get_height();
            let format = &dds.header.spf;

            // Check for uncompressed RGB/RGBA (Flags 0x40 = RGB, 0x41 = RGBA)
            if format.flags.contains(ddsfile::PixelFormatFlags::RGB) {
                let bit_count = format.rgb_bit_count;
                let data = &dds.data;
                
                match bit_count {
                    Some(32) => {
                        // Likely B8G8R8A8 or R8G8B8A8
                        let mut rgba = Vec::with_capacity((width * height * 4) as usize);
                        for i in (0..data.len()).step_by(4) {
                            if i + 3 < data.len() {
                                rgba.push(data[i+2]); // R
                                rgba.push(data[i+1]); // G
                                rgba.push(data[i]);   // B
                                rgba.push(data[i+3]); // A
                            }
                        }
                        if let Some(buf) = ImageBuffer::<Rgba<u8>, _>::from_raw(width, height, rgba) {
                            return Ok(DynamicImage::ImageRgba8(buf));
                        }
                    },
                    Some(24) => {
                        let mut rgb = Vec::with_capacity((width * height * 3) as usize);
                        for i in (0..data.len()).step_by(3) {
                            if i + 2 < data.len() {
                                rgb.push(data[i+2]); // R
                                rgb.push(data[i+1]); // G
                                rgb.push(data[i]);   // B
                            }
                        }
                        if let Some(buf) = ImageBuffer::<Rgb<u8>, _>::from_raw(width, height, rgb) {
                            return Ok(DynamicImage::ImageRgb8(buf));
                        }
                    },
                    _ => {}
                }
            }
            
            Err(anyhow::anyhow!("Unsupported DDS format or corrupted file"))
        }
    }
}

fn load_image_any(path: &str) -> Result<DynamicImage> {
    let ext = Path::new(path).extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    
    if ext == "ddj" || ext == "dds" {
        read_dds_content(path)
    } else {
        Ok(image::open(path)?)
    }
}

#[tauri::command]
async fn get_image_preview(path: String) -> Result<ImageMetadata, String> {
    let img = load_image_any(&path).map_err(|e| e.to_string())?;
    
    // Create a small preview
    let preview_img = img.thumbnail(128, 128);
    let mut buffer = Cursor::new(Vec::new());
    preview_img.write_to(&mut buffer, ImageOutputFormat::Png).map_err(|e| e.to_string())?;
    
    let base64_image = general_purpose::STANDARD.encode(buffer.into_inner());
    
    Ok(ImageMetadata {
        name: Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or("unknown").to_string(),
        extension: Path::new(&path).extension().and_then(|s| s.to_str()).unwrap_or("").to_string(),
        width: img.width(),
        height: img.height(),
        preview: format!("data:image/png;base64,{}", base64_image),
    })
}

#[tauri::command]
async fn convert_image(path: String, target_format: String, save_dir: String) -> ConversionResult {
    let filename = Path::new(&path).file_stem().and_then(|s| s.to_str()).unwrap_or("unknown");
    let target_path = Path::new(&save_dir).join(format!("{}.{}", filename, target_format.to_lowercase()));
    
    let result = (|| -> Result<()> {
        let img = load_image_any(&path)?;
        let rgba = img.to_rgba8();
        
        match target_format.to_lowercase().as_str() {
            "ddj" => {
                let dds = image_dds::dds_from_image(&rgba, image_dds::ImageFormat::BC3Unorm, image_dds::Quality::Normal, image_dds::Mipmaps::GeneratedAutomatic)?;
                let mut dds_data = Vec::new();
                dds.write(&mut dds_data)?;
                
                let mut file = File::create(&target_path)?;
                file.write_all(&DDJ_HEADER)?;
                file.write_all(&dds_data)?;
            },
            "dds" => {
                let dds = image_dds::dds_from_image(&rgba, image_dds::ImageFormat::BC3Unorm, image_dds::Quality::Normal, image_dds::Mipmaps::GeneratedAutomatic)?;
                let mut file = File::create(&target_path)?;
                dds.write(&mut file)?;
            },
            "png" => img.save_with_format(&target_path, image::ImageFormat::Png)?,
            "jpg" | "jpeg" => img.save_with_format(&target_path, image::ImageFormat::Jpeg)?,
            "bmp" => img.save_with_format(&target_path, image::ImageFormat::Bmp)?,
            "gif" => img.save_with_format(&target_path, image::ImageFormat::Gif)?,
            "tif" | "tiff" => img.save_with_format(&target_path, image::ImageFormat::Tiff)?,
            "tga" => img.save_with_format(&target_path, image::ImageFormat::Tga)?,
            "ico" => {
                // Resize if needed, ICO supports max 256x256
                let final_img = if img.width() > 256 || img.height() > 256 {
                    img.thumbnail(256, 256)
                } else {
                    img
                };
                final_img.save_with_format(&target_path, image::ImageFormat::Ico)?
            },
            _ => return Err(anyhow::anyhow!("Unsupported target format")),
        }
        Ok(())
    })();

    match result {
        Ok(_) => ConversionResult { success: true, filename: filename.to_string(), error: None },
        Err(e) => ConversionResult { success: false, filename: filename.to_string(), error: Some(e.to_string()) },
    }
}

#[tauri::command]
async fn read_folder(path: String) -> Result<Vec<String>, String> {
    let mut files = Vec::new();
    let mut dirs = vec![std::path::PathBuf::from(path)];

    while let Some(dir) = dirs.pop() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if ["png", "jpg", "jpeg", "bmp", "dds", "ddj", "tif", "tiff", "gif"].contains(&ext_lower.as_str()) {
                        if let Some(p) = path.to_str() {
                            files.push(p.to_string());
                        }
                    }
                }
            }
        }
    }
    Ok(files)
}

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err("Not implemented for this OS".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_image_preview, convert_image, read_folder, open_folder])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
