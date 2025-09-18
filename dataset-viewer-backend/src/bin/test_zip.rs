use dataset_viewer_backend::archive::{ArchiveHandler, ArchiveFormat};
use std::fs::File;
use std::io::BufReader;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing ZIP archive functionality...");

    // 测试ZIP文件路径
    let zip_path = "D:\\git-ai\\dataset-viewer\\test_archive.zip";

    // 检查文件是否存在
    if !std::path::Path::new(zip_path).exists() {
        eprintln!("Test ZIP file not found: {}", zip_path);
        return Ok(());
    }

    // 打开ZIP文件
    let file = File::open(zip_path)?;
    let reader = BufReader::new(file);

    println!("Analyzing ZIP archive...");

    // 分析ZIP压缩包
    match ArchiveHandler::analyze(reader, ArchiveFormat::Zip, Some(100)).await {
        Ok(archive_info) => {
            println!("✅ ZIP analysis successful!");
            println!("Total entries: {}", archive_info.total_entries);
            println!("Total uncompressed size: {} bytes", archive_info.total_uncompressed_size);
            println!("Total compressed size: {} bytes", archive_info.total_compressed_size);
            println!("Format: {:?}", archive_info.format);
            println!("Has more: {}", archive_info.has_more);

            println!("\nEntries:");
            for entry in &archive_info.entries {
                println!("  📁 {}", entry.path);
                println!("     Name: {}", entry.name);
                println!("     Size: {} bytes", entry.size);
                if let Some(compressed) = entry.compressed_size {
                    println!("     Compressed: {} bytes", compressed);
                }
                if let Some(modified) = &entry.modified {
                    println!("     Modified: {}", modified);
                }
                println!("     Directory: {}", entry.is_directory);
                println!("     Encrypted: {}", entry.is_encrypted);
                if let Some(crc) = entry.crc32 {
                    println!("     CRC32: {:08x}", crc);
                }
                println!();
            }
        }
        Err(e) => {
            eprintln!("❌ ZIP analysis failed: {}", e);
            return Err(e.into());
        }
    }

    // 测试文件提取
    println!("Testing file extraction...");
    let file = File::open(zip_path)?;
    let reader = BufReader::new(file);

    match ArchiveHandler::extract_file(reader, ArchiveFormat::Zip, "test.txt", Some(1024)).await {
        Ok(file_preview) => {
            println!("✅ File extraction successful!");
            println!("Total size: {} bytes", file_preview.total_size);
            println!("Preview size: {} bytes", file_preview.preview_size);
            println!("Is truncated: {}", file_preview.is_truncated);
            println!("Content: {}", String::from_utf8_lossy(&file_preview.content));
        }
        Err(e) => {
            eprintln!("❌ File extraction failed: {}", e);
            return Err(e.into());
        }
    }

    println!("All tests passed! 🎉");
    Ok(())
}