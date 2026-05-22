use std::{fs::File, io::Read, path::Path};

const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

pub fn inspect_png(path: &Path) -> Result<PngInspection, String> {
    let mut file = File::open(path).map_err(|error| format!("打开文件失败: {error}"))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("读取文件失败: {error}"))?;

    if bytes.len() < 8 || bytes[0..8] != PNG_SIGNATURE {
        return Err("不是有效的 PNG 文件".to_string());
    }

    let mut offset = 8usize;
    let mut has_ihdr = false;
    let mut is_apng = false;

    while offset + 8 <= bytes.len() {
        let length = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]) as usize;

        if offset + 12 + length > bytes.len() {
            return Err("PNG 结构不完整".to_string());
        }

        let chunk_type = &bytes[offset + 4..offset + 8];
        match chunk_type {
            b"IHDR" => has_ihdr = true,
            b"acTL" => is_apng = true,
            _ => {}
        }

        offset += 12 + length;
        if chunk_type == b"IEND" {
            break;
        }
    }

    if !has_ihdr {
        return Err("PNG 缺少 IHDR 头".to_string());
    }

    Ok(PngInspection { is_apng })
}

pub struct PngInspection {
    pub is_apng: bool,
}
