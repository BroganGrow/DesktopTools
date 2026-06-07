use image::{DynamicImage, GenericImageView, RgbaImage};
use resvg::{
    tiny_skia::{Color, Pixmap, Transform},
    usvg,
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::AppHandle;
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImagesToPdfRequest {
    pub image_paths: Vec<String>,
    pub output_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfToImagesRequest {
    pub pdf_path: String,
    pub output_dir: String,
    pub format: PdfImageFormat,
    pub dpi: u16,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MergePdfsRequest {
    pub pdf_paths: Vec<String>,
    pub output_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitPdfRequest {
    pub pdf_path: String,
    pub output_dir: String,
    pub mode: SplitMode,
    pub page_ranges: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatermarkPdfRequest {
    pub pdf_path: String,
    pub output_path: String,
    pub text: String,
    pub font_size: f32,
    pub opacity: f32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PdfImageFormat {
    Png,
    Jpg,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SplitMode {
    Range,
    Pages,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PdfOperationResult {
    pub output_dir: String,
    pub output_paths: Vec<String>,
    pub message: String,
}

pub async fn images_to_pdf_file(
    app: AppHandle,
    request: ImagesToPdfRequest,
) -> Result<PdfOperationResult, String> {
    if request.image_paths.is_empty() {
        return Err("请选择至少一张图片。".to_string());
    }

    let output_path = normalize_pdf_output_path(&request.output_path);
    ensure_parent_dir(&output_path)?;
    let temp_dir = temp_work_dir("images-to-pdf")?;
    fs::create_dir_all(&temp_dir).map_err(|error| format!("创建临时目录失败: {error}"))?;
    let raw_output = temp_dir.join("raw.pdf");
    write_images_pdf(&request.image_paths, &raw_output)?;
    run_mutool(
        &app,
        vec![
            "clean".to_string(),
            "-gggg".to_string(),
            "-z".to_string(),
            "-i".to_string(),
            "-e".to_string(),
            "80".to_string(),
            raw_output.display().to_string(),
            output_path.display().to_string(),
        ],
    )
    .await?;
    let _ = fs::remove_dir_all(&temp_dir);

    Ok(single_output_result(
        &output_path,
        format!(
            "已生成 1 个 PDF，包含 {} 张图片。",
            request.image_paths.len()
        ),
    ))
}

pub async fn pdf_to_images_file(
    app: AppHandle,
    request: PdfToImagesRequest,
) -> Result<PdfOperationResult, String> {
    let pdf_path = PathBuf::from(&request.pdf_path);
    ensure_file(&pdf_path, "PDF 文件")?;
    let output_dir = PathBuf::from(&request.output_dir)
        .join(format!("{}_images", file_stem_or_default(&pdf_path, "pdf")));
    fs::create_dir_all(&output_dir).map_err(|error| format!("创建输出目录失败: {error}"))?;

    let output_paths = render_pdf_to_images(
        &app,
        &pdf_path,
        &output_dir,
        request.dpi.clamp(36, 600),
        &request.format,
    )
    .await?;

    Ok(PdfOperationResult {
        output_dir: output_dir.display().to_string(),
        message: format!("已导出 {} 张图片。", output_paths.len()),
        output_paths,
    })
}

pub async fn merge_pdfs_file(
    app: AppHandle,
    request: MergePdfsRequest,
) -> Result<PdfOperationResult, String> {
    if request.pdf_paths.len() < 2 {
        return Err("请选择至少两个 PDF。".to_string());
    }

    let output_path = normalize_pdf_output_path(&request.output_path);
    ensure_parent_dir(&output_path)?;

    let mut args = vec![
        "merge".to_string(),
        "-o".to_string(),
        output_path.display().to_string(),
        "-O".to_string(),
        "garbage=deduplicate,compress=yes,compress-images".to_string(),
    ];

    for path in &request.pdf_paths {
        ensure_file(Path::new(path), "PDF 文件")?;
        args.push(path.clone());
    }

    run_mutool(&app, args).await?;
    Ok(single_output_result(
        &output_path,
        format!("已合并 {} 个 PDF。", request.pdf_paths.len()),
    ))
}

pub async fn split_pdf_file(
    app: AppHandle,
    request: SplitPdfRequest,
) -> Result<PdfOperationResult, String> {
    let pdf_path = PathBuf::from(&request.pdf_path);
    ensure_file(&pdf_path, "PDF 文件")?;
    let output_dir = PathBuf::from(&request.output_dir)
        .join(format!("{}_split", file_stem_or_default(&pdf_path, "pdf")));
    fs::create_dir_all(&output_dir).map_err(|error| format!("创建输出目录失败: {error}"))?;

    let output_paths = match request.mode {
        SplitMode::Range => {
            let ranges = request.page_ranges.trim();
            if ranges.is_empty() {
                return Err("请输入页码范围，例如 1-3,5。".to_string());
            }
            let output_path = output_dir.join(format!(
                "{}_selected.pdf",
                file_stem_or_default(&pdf_path, "pdf")
            ));
            run_mutool(
                &app,
                vec![
                    "merge".to_string(),
                    "-o".to_string(),
                    output_path.display().to_string(),
                    pdf_path.display().to_string(),
                    ranges.to_string(),
                ],
            )
            .await?;
            vec![output_path.display().to_string()]
        }
        SplitMode::Pages => {
            let page_count = get_pdf_page_count(&app, &pdf_path).await?;
            let mut outputs = Vec::new();
            for page in 1..=page_count {
                let output_path = output_dir.join(format!("page-{page:03}.pdf"));
                run_mutool(
                    &app,
                    vec![
                        "merge".to_string(),
                        "-o".to_string(),
                        output_path.display().to_string(),
                        pdf_path.display().to_string(),
                        page.to_string(),
                    ],
                )
                .await?;
                outputs.push(output_path.display().to_string());
            }
            outputs
        }
    };

    Ok(PdfOperationResult {
        output_dir: output_dir.display().to_string(),
        message: format!("已拆分输出 {} 个文件。", output_paths.len()),
        output_paths,
    })
}

pub async fn watermark_pdf_file(
    app: AppHandle,
    request: WatermarkPdfRequest,
) -> Result<PdfOperationResult, String> {
    if request.text.trim().is_empty() {
        return Err("请输入水印文本。".to_string());
    }

    let pdf_path = PathBuf::from(&request.pdf_path);
    ensure_file(&pdf_path, "PDF 文件")?;
    let output_path = normalize_pdf_output_path(&request.output_path);
    ensure_parent_dir(&output_path)?;

    let temp_dir = temp_work_dir("pdf-watermark")?;
    fs::create_dir_all(&temp_dir).map_err(|error| format!("创建临时目录失败: {error}"))?;
    let rendered_dir = temp_dir.join("pages");
    fs::create_dir_all(&rendered_dir).map_err(|error| format!("创建临时页面目录失败: {error}"))?;

    let rendered_pages =
        render_pdf_to_images(&app, &pdf_path, &rendered_dir, 144, &PdfImageFormat::Png).await?;

    let mut watermarked_paths = Vec::new();
    for (index, page_path) in rendered_pages.iter().enumerate() {
        let page = image::open(page_path)
            .map_err(|error| format!("读取页面图片失败: {error}"))?
            .to_rgba8();
        let watermarked = apply_text_watermark(page, &request)?;
        let target = temp_dir.join(format!("watermarked-{index:04}.png"));
        watermarked
            .save(&target)
            .map_err(|error| format!("保存水印页面失败: {error}"))?;
        watermarked_paths.push(target.display().to_string());
    }

    let raw_output = temp_dir.join("watermarked_raw.pdf");
    write_images_pdf(&watermarked_paths, &raw_output)?;
    run_mutool(
        &app,
        vec![
            "clean".to_string(),
            "-gggg".to_string(),
            "-z".to_string(),
            "-i".to_string(),
            "-e".to_string(),
            "80".to_string(),
            raw_output.display().to_string(),
            output_path.display().to_string(),
        ],
    )
    .await?;
    let _ = fs::remove_dir_all(&temp_dir);

    Ok(single_output_result(
        &output_path,
        "已添加文本水印。".to_string(),
    ))
}

async fn render_pdf_to_images(
    app: &AppHandle,
    pdf_path: &Path,
    output_dir: &Path,
    dpi: u16,
    format: &PdfImageFormat,
) -> Result<Vec<String>, String> {
    fs::create_dir_all(output_dir).map_err(|error| format!("创建输出目录失败: {error}"))?;
    let pattern = output_dir.join("page-%04d.png");
    run_mutool(
        app,
        vec![
            "draw".to_string(),
            "-q".to_string(),
            "-r".to_string(),
            dpi.to_string(),
            "-F".to_string(),
            "png".to_string(),
            "-o".to_string(),
            pattern.display().to_string(),
            pdf_path.display().to_string(),
        ],
    )
    .await?;

    let pngs = collect_outputs(output_dir, "png")?;
    if matches!(format, PdfImageFormat::Png) {
        return Ok(pngs);
    }

    let mut jpgs = Vec::new();
    for png in pngs {
        let source = PathBuf::from(&png);
        let stem = source
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("page");
        let target = output_dir.join(format!("{stem}.jpg"));
        image::open(&source)
            .map_err(|error| format!("读取渲染图片失败: {error}"))?
            .to_rgb8()
            .save_with_format(&target, image::ImageFormat::Jpeg)
            .map_err(|error| format!("写入 JPG 失败: {error}"))?;
        let _ = fs::remove_file(&source);
        jpgs.push(target.display().to_string());
    }
    Ok(jpgs)
}

async fn get_pdf_page_count(app: &AppHandle, pdf_path: &Path) -> Result<usize, String> {
    let output = run_mutool_output(
        app,
        vec!["pages".to_string(), pdf_path.display().to_string()],
    )
    .await?;
    let count = output
        .lines()
        .filter(|line| line.trim_start().starts_with("<page "))
        .count();
    if count == 0 {
        Err("无法读取 PDF 页数。".to_string())
    } else {
        Ok(count)
    }
}

async fn run_mutool(app: &AppHandle, args: Vec<String>) -> Result<(), String> {
    run_mutool_output(app, args).await.map(|_| ())
}

async fn run_mutool_output(app: &AppHandle, args: Vec<String>) -> Result<String, String> {
    let command = app
        .shell()
        .sidecar("mutool")
        .map_err(|error| format!("未找到 mutool sidecar: {error}"))?;
    let output = command
        .args(args)
        .output()
        .await
        .map_err(|error| format!("执行 mutool 失败: {error}"))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if stderr.is_empty() {
        "mutool 返回失败状态。".to_string()
    } else {
        stderr
    })
}

fn write_images_pdf(image_paths: &[String], output_path: &Path) -> Result<(), String> {
    let mut objects: Vec<Vec<u8>> = vec![Vec::new(), Vec::new()];
    let mut page_ids = Vec::new();

    for path in image_paths {
        let image = image::open(path).map_err(|error| format!("读取图片失败: {path} ({error})"))?;
        let (width, height, rgb) = flatten_image_to_rgb(image);

        let image_id = objects.len() + 1;
        objects.push(pdf_stream_object(
            format!(
                "<< /Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Length {} >>",
                rgb.len()
            ),
            &rgb,
        ));

        let content = format!("q\n{width} 0 0 {height} 0 0 cm\n/Im1 Do\nQ\n");
        let content_id = objects.len() + 1;
        objects.push(pdf_stream_object(
            format!("<< /Length {} >>", content.len()),
            content.as_bytes(),
        ));

        let page_id = objects.len() + 1;
        page_ids.push(page_id);
        objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {width} {height}] /Resources << /XObject << /Im1 {image_id} 0 R >> >> /Contents {content_id} 0 R >>"
            )
            .into_bytes(),
        );
    }

    objects[0] = b"<< /Type /Catalog /Pages 2 0 R >>".to_vec();
    let kids = page_ids
        .iter()
        .map(|id| format!("{id} 0 R"))
        .collect::<Vec<_>>()
        .join(" ");
    objects[1] = format!(
        "<< /Type /Pages /Kids [{kids}] /Count {} >>",
        page_ids.len()
    )
    .into_bytes();

    write_pdf_objects(&objects, output_path)
}

fn write_pdf_objects(objects: &[Vec<u8>], output_path: &Path) -> Result<(), String> {
    let file = fs::File::create(output_path).map_err(|error| format!("创建 PDF 失败: {error}"))?;
    let mut writer = BufWriter::new(file);
    let mut offsets = Vec::with_capacity(objects.len());
    let mut position = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".len();

    writer
        .write_all(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n")
        .map_err(|error| format!("写入 PDF 失败: {error}"))?;

    for (index, object) in objects.iter().enumerate() {
        offsets.push(position);
        let header = format!("{} 0 obj\n", index + 1);
        writer
            .write_all(header.as_bytes())
            .and_then(|_| writer.write_all(object))
            .and_then(|_| writer.write_all(b"\nendobj\n"))
            .map_err(|error| format!("写入 PDF 对象失败: {error}"))?;
        position += header.len() + object.len() + b"\nendobj\n".len();
    }

    let xref_position = position;
    let mut xref = format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1);
    for offset in offsets {
        xref.push_str(&format!("{offset:010} 00000 n \n"));
    }
    xref.push_str(&format!(
        "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref_position}\n%%EOF\n",
        objects.len() + 1
    ));

    writer
        .write_all(xref.as_bytes())
        .map_err(|error| format!("写入 PDF 交叉引用失败: {error}"))
}

fn pdf_stream_object(dictionary: String, data: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    output.extend_from_slice(dictionary.as_bytes());
    output.extend_from_slice(b"\nstream\n");
    output.extend_from_slice(data);
    output.extend_from_slice(b"\nendstream");
    output
}

fn flatten_image_to_rgb(image: DynamicImage) -> (u32, u32, Vec<u8>) {
    let (width, height) = image.dimensions();
    let rgba = image.to_rgba8();
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    for pixel in rgba.pixels() {
        let [r, g, b, a] = pixel.0;
        if a == 255 {
            rgb.extend_from_slice(&[r, g, b]);
            continue;
        }
        let alpha = a as u16;
        let inv_alpha = 255u16.saturating_sub(alpha);
        rgb.push((((r as u16 * alpha) + (255 * inv_alpha)) / 255) as u8);
        rgb.push((((g as u16 * alpha) + (255 * inv_alpha)) / 255) as u8);
        rgb.push((((b as u16 * alpha) + (255 * inv_alpha)) / 255) as u8);
    }
    (width, height, rgb)
}

fn apply_text_watermark(
    mut page: RgbaImage,
    request: &WatermarkPdfRequest,
) -> Result<RgbaImage, String> {
    let watermark = render_watermark_svg(
        page.width(),
        page.height(),
        request.text.trim(),
        request.font_size,
        request.opacity,
    )?;
    let watermark_data = watermark.data();

    for (index, pixel) in page.pixels_mut().enumerate() {
        let base = pixel.0;
        let offset = index * 4;
        let mark = [
            watermark_data[offset],
            watermark_data[offset + 1],
            watermark_data[offset + 2],
            watermark_data[offset + 3],
        ];
        let alpha = mark[3] as f32 / 255.0;
        if alpha <= 0.0 {
            continue;
        }
        pixel.0 = [
            blend_channel(base[0], mark[0], alpha),
            blend_channel(base[1], mark[1], alpha),
            blend_channel(base[2], mark[2], alpha),
            255,
        ];
    }

    Ok(page)
}

fn blend_channel(base: u8, mark: u8, alpha: f32) -> u8 {
    ((base as f32 * (1.0 - alpha)) + (mark as f32 * alpha)).round() as u8
}

fn render_watermark_svg(
    width: u32,
    height: u32,
    text: &str,
    font_size: f32,
    opacity: f32,
) -> Result<Pixmap, String> {
    let mut pixmap = Pixmap::new(width, height).ok_or_else(|| "无法创建水印画布".to_string())?;
    pixmap.fill(Color::from_rgba8(255, 255, 255, 0));

    let svg = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{width}" height="{height}" viewBox="0 0 {width} {height}">
  <g transform="translate({cx} {cy}) rotate(-35)">
    <text x="0" y="0" text-anchor="middle" dominant-baseline="middle" font-family="Arial, Helvetica, sans-serif" font-size="{font_size}" font-weight="700" fill="#111111" fill-opacity="{opacity}">{text}</text>
  </g>
</svg>"##,
        cx = width as f32 / 2.0,
        cy = height as f32 / 2.0,
        font_size = font_size.clamp(12.0, 160.0),
        opacity = opacity.clamp(0.05, 1.0),
        text = escape_xml_text(text)
    );
    let options = usvg::Options::default();
    let tree = usvg::Tree::from_str(&svg, &options)
        .map_err(|error| format!("生成水印 SVG 失败: {error}"))?;
    resvg::render(&tree, Transform::identity(), &mut pixmap.as_mut());
    Ok(pixmap)
}

fn escape_xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn collect_outputs(output_dir: &Path, extension: &str) -> Result<Vec<String>, String> {
    let mut outputs = fs::read_dir(output_dir)
        .map_err(|error| format!("读取输出目录失败: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(|value| value.to_str())
                .map(|value| value.eq_ignore_ascii_case(extension))
                .unwrap_or(false)
        })
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();
    outputs.sort();
    Ok(outputs)
}

fn single_output_result(output_path: &Path, message: String) -> PdfOperationResult {
    PdfOperationResult {
        output_dir: output_path
            .parent()
            .map(|path| path.display().to_string())
            .unwrap_or_default(),
        output_paths: vec![output_path.display().to_string()],
        message,
    }
}

fn temp_work_dir(prefix: &str) -> Result<PathBuf, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    Ok(std::env::temp_dir().join(format!("supertools-{prefix}-{stamp}")))
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "无法确定输出目录。".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("创建输出目录失败: {error}"))
}

fn ensure_file(path: &Path, label: &str) -> Result<(), String> {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("{label}不存在: {}", path.display()))
    }
}

fn normalize_pdf_output_path(path: &str) -> PathBuf {
    let mut output = PathBuf::from(path);
    if output.extension().is_none() {
        output.set_extension("pdf");
    }
    output
}

fn file_stem_or_default(path: &Path, fallback: &str) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(sanitize_file_stem)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn sanitize_file_stem(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '_'
            }
        })
        .collect()
}
