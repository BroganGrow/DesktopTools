use image::{codecs::jpeg::JpegEncoder, ImageBuffer, RgbaImage};
use resvg::{
    tiny_skia::{Color, Pixmap, Transform},
    usvg,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgExportRequest {
    pub svg_code: String,
    pub width: u32,
    pub height: u32,
    pub format: SvgExportFormat,
    pub quality: u8,
    pub keep_aspect_ratio: bool,
    pub background: String,
    pub output_path: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SvgExportFormat {
    Png,
    Jpg,
    Jpeg,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgExportResult {
    pub output_path: String,
    pub output_dir: String,
    pub width: u32,
    pub height: u32,
    pub format: SvgExportFormat,
    pub output_bytes: u64,
}

pub fn export_svg_image_file(request: SvgExportRequest) -> Result<SvgExportResult, String> {
    validate_request(&request)?;

    let output_path = PathBuf::from(&request.output_path);
    let output_dir = output_path
        .parent()
        .ok_or_else(|| "输出路径缺少父目录".to_string())?;
    std::fs::create_dir_all(output_dir).map_err(|error| format!("创建输出目录失败: {error}"))?;

    let mut options = usvg::Options::default();
    options.resources_dir = output_dir.parent().map(Path::to_path_buf);

    let tree =
        usvg::Tree::from_str(&request.svg_code, &options).map_err(|error| format!("SVG 解析失败: {error}"))?;

    let mut pixmap =
        Pixmap::new(request.width, request.height).ok_or_else(|| "无法创建目标画布".to_string())?;

    let clear_color = match request.format {
        SvgExportFormat::Png => parse_color(&request.background).unwrap_or_else(|| Color::from_rgba8(255, 255, 255, 0)),
        SvgExportFormat::Jpg | SvgExportFormat::Jpeg => {
            parse_color(&request.background).unwrap_or_else(|| Color::from_rgba8(255, 255, 255, 255))
        }
    };
    pixmap.fill(clear_color);

    let svg_size = tree.size();
    let source_width = svg_size.width();
    let source_height = svg_size.height();

    let transform = if request.keep_aspect_ratio {
        let scale = f32::min(request.width as f32 / source_width, request.height as f32 / source_height);
        let scaled_width = source_width * scale;
        let scaled_height = source_height * scale;
        let tx = (request.width as f32 - scaled_width) / 2.0;
        let ty = (request.height as f32 - scaled_height) / 2.0;
        Transform::from_scale(scale, scale).post_translate(tx, ty)
    } else {
        Transform::from_scale(request.width as f32 / source_width, request.height as f32 / source_height)
    };

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    save_pixmap(&pixmap, &output_path, &request)?;

    let output_bytes = std::fs::metadata(&output_path)
        .map_err(|error| format!("读取输出文件信息失败: {error}"))?
        .len();

    Ok(SvgExportResult {
        output_path: output_path.display().to_string(),
        output_dir: output_dir.display().to_string(),
        width: request.width,
        height: request.height,
        format: request.format,
        output_bytes,
    })
}

fn validate_request(request: &SvgExportRequest) -> Result<(), String> {
    if request.svg_code.trim().is_empty() {
        return Err("SVG 代码不能为空".to_string());
    }

    if request.width == 0 || request.height == 0 {
        return Err("导出尺寸必须大于 0".to_string());
    }

    if !request.output_path.trim().is_empty() {
        return Ok(());
    }

    Err("缺少输出路径".to_string())
}

fn save_pixmap(pixmap: &Pixmap, output_path: &Path, request: &SvgExportRequest) -> Result<(), String> {
    match request.format {
        SvgExportFormat::Png => pixmap
            .save_png(output_path)
            .map_err(|error| format!("写入 PNG 失败: {error}")),
        SvgExportFormat::Jpg | SvgExportFormat::Jpeg => {
            let image = rgba_image_from_pixmap(pixmap)?;
            let writer = BufWriter::new(File::create(output_path).map_err(|error| format!("创建 JPG 文件失败: {error}"))?);
            let mut encoder = JpegEncoder::new_with_quality(writer, request.quality.clamp(1, 100));
            encoder
                .encode_image(&image)
                .map_err(|error| format!("写入 JPG 失败: {error}"))
        }
    }
}

fn rgba_image_from_pixmap(pixmap: &Pixmap) -> Result<RgbaImage, String> {
    ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
        .ok_or_else(|| "无法从 SVG 渲染结果构建图像缓冲".to_string())
}

fn parse_color(input: &str) -> Option<Color> {
    let raw = input.trim().trim_start_matches('#');
    match raw.len() {
        6 => {
            let red = u8::from_str_radix(&raw[0..2], 16).ok()?;
            let green = u8::from_str_radix(&raw[2..4], 16).ok()?;
            let blue = u8::from_str_radix(&raw[4..6], 16).ok()?;
            Some(Color::from_rgba8(red, green, blue, 255))
        }
        8 => {
            let red = u8::from_str_radix(&raw[0..2], 16).ok()?;
            let green = u8::from_str_radix(&raw[2..4], 16).ok()?;
            let blue = u8::from_str_radix(&raw[4..6], 16).ok()?;
            let alpha = u8::from_str_radix(&raw[6..8], 16).ok()?;
            Some(Color::from_rgba8(red, green, blue, alpha))
        }
        _ => None,
    }
}
