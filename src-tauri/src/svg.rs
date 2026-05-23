use image::{codecs::jpeg::JpegEncoder, DynamicImage, ImageBuffer, RgbaImage};
use resvg::{
    tiny_skia::{Color, Pixmap, Transform},
    usvg,
};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufWriter, Cursor},
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
    pub padding_percent: u8,
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgIconSetRequest {
    pub svg_code: String,
    pub output_dir: String,
    pub app_name: String,
    pub platforms: Vec<IconPlatform>,
    pub background: String,
    pub padding_percent: u8,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum IconPlatform {
    Android,
    Ios,
    Flutter,
    Electron,
    Tauri,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IconAssetResult {
    pub platform: IconPlatform,
    pub output_path: String,
    pub width: u32,
    pub height: u32,
    pub output_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SvgIconSetResult {
    pub output_dir: String,
    pub generated_count: usize,
    pub total_bytes: u64,
    pub platforms: Vec<IconPlatform>,
    pub assets: Vec<IconAssetResult>,
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

    let tree = usvg::Tree::from_str(&request.svg_code, &options)
        .map_err(|error| format!("SVG 解析失败: {error}"))?;

    let clear_color = match request.format {
        SvgExportFormat::Png => {
            parse_color(&request.background).unwrap_or_else(|| Color::from_rgba8(255, 255, 255, 0))
        }
        SvgExportFormat::Jpg | SvgExportFormat::Jpeg => parse_color(&request.background)
            .unwrap_or_else(|| Color::from_rgba8(255, 255, 255, 255)),
    };
    let pixmap = render_tree_pixmap_with_color(
        &tree,
        request.width,
        request.height,
        clear_color,
        request.keep_aspect_ratio,
        request.padding_percent,
    )?;

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

pub fn export_svg_icon_set(request: SvgIconSetRequest) -> Result<SvgIconSetResult, String> {
    validate_icon_set_request(&request)?;

    let base_dir = PathBuf::from(&request.output_dir);
    let package_name = sanitize_file_stem(&request.app_name);
    let output_root = base_dir.join(format!("{package_name}_icons"));
    std::fs::create_dir_all(&output_root).map_err(|error| format!("创建输出目录失败: {error}"))?;

    let source_svg_path = output_root.join("source.svg");
    std::fs::write(&source_svg_path, &request.svg_code)
        .map_err(|error| format!("写入源 SVG 失败: {error}"))?;

    let mut options = usvg::Options::default();
    options.resources_dir = Some(output_root.clone());
    let tree = usvg::Tree::from_str(&request.svg_code, &options)
        .map_err(|error| format!("SVG 解析失败: {error}"))?;

    let mut assets = Vec::new();

    for platform in &request.platforms {
        match platform {
            IconPlatform::Android => generate_android_icons(
                &tree,
                &output_root.join("android"),
                IconPlatform::Android,
                &request,
                &mut assets,
            )?,
            IconPlatform::Ios => generate_ios_icons(
                &tree,
                &output_root.join("ios"),
                IconPlatform::Ios,
                &request,
                &mut assets,
            )?,
            IconPlatform::Flutter => {
                generate_flutter_icons(&tree, &output_root.join("flutter"), &request, &mut assets)?
            }
            IconPlatform::Electron => generate_electron_icons(
                &tree,
                &output_root.join("electron"),
                IconPlatform::Electron,
                &request,
                &mut assets,
            )?,
            IconPlatform::Tauri => generate_tauri_icons(
                &tree,
                &output_root.join("tauri"),
                IconPlatform::Tauri,
                &request,
                &mut assets,
            )?,
        }
    }

    let total_bytes = assets.iter().map(|asset| asset.output_bytes).sum();

    Ok(SvgIconSetResult {
        output_dir: output_root.display().to_string(),
        generated_count: assets.len(),
        total_bytes,
        platforms: request.platforms,
        assets,
    })
}

fn validate_request(request: &SvgExportRequest) -> Result<(), String> {
    if request.svg_code.trim().is_empty() {
        return Err("SVG 代码不能为空".to_string());
    }

    if request.width == 0 || request.height == 0 {
        return Err("导出尺寸必须大于 0".to_string());
    }

    if request.padding_percent > 40 {
        return Err("图标内边距不能超过 40%".to_string());
    }

    if !request.output_path.trim().is_empty() {
        return Ok(());
    }

    Err("缺少输出路径".to_string())
}

fn validate_icon_set_request(request: &SvgIconSetRequest) -> Result<(), String> {
    if request.svg_code.trim().is_empty() {
        return Err("SVG 代码不能为空".to_string());
    }

    if request.output_dir.trim().is_empty() {
        return Err("请选择输出目录".to_string());
    }

    if request.platforms.is_empty() {
        return Err("请至少选择一个目标平台".to_string());
    }

    if request.padding_percent > 40 {
        return Err("图标内边距不能超过 40%".to_string());
    }

    Ok(())
}

fn save_pixmap(
    pixmap: &Pixmap,
    output_path: &Path,
    request: &SvgExportRequest,
) -> Result<(), String> {
    match request.format {
        SvgExportFormat::Png => pixmap
            .save_png(output_path)
            .map_err(|error| format!("写入 PNG 失败: {error}")),
        SvgExportFormat::Jpg | SvgExportFormat::Jpeg => {
            let image = rgba_image_from_pixmap(pixmap)?;
            let writer = BufWriter::new(
                File::create(output_path).map_err(|error| format!("创建 JPG 文件失败: {error}"))?,
            );
            let mut encoder = JpegEncoder::new_with_quality(writer, request.quality.clamp(1, 100));
            encoder
                .encode_image(&image)
                .map_err(|error| format!("写入 JPG 失败: {error}"))
        }
    }
}

fn generate_android_icons(
    tree: &usvg::Tree,
    root: &Path,
    platform: IconPlatform,
    request: &SvgIconSetRequest,
    assets: &mut Vec<IconAssetResult>,
) -> Result<(), String> {
    let densities = [
        ("mipmap-mdpi", 48, 108),
        ("mipmap-hdpi", 72, 162),
        ("mipmap-xhdpi", 96, 216),
        ("mipmap-xxhdpi", 144, 324),
        ("mipmap-xxxhdpi", 192, 432),
    ];

    for (folder, legacy_size, foreground_size) in densities {
        let legacy_rel = format!("app/src/main/res/{folder}/ic_launcher.png");
        let round_rel = format!("app/src/main/res/{folder}/ic_launcher_round.png");
        let foreground_rel = format!("app/src/main/res/{folder}/ic_launcher_foreground.png");
        assets.push(write_png_asset(
            tree,
            root,
            platform,
            &legacy_rel,
            legacy_size,
            legacy_size,
            &request.background,
            request.padding_percent,
        )?);
        assets.push(write_png_asset(
            tree,
            root,
            platform,
            &round_rel,
            legacy_size,
            legacy_size,
            &request.background,
            request.padding_percent,
        )?);
        assets.push(write_png_asset(
            tree,
            root,
            platform,
            &foreground_rel,
            foreground_size,
            foreground_size,
            "#00000000",
            request.padding_percent.max(17),
        )?);
    }

    assets.push(write_png_asset(
        tree,
        root,
        platform,
        "play-store-icon.png",
        512,
        512,
        &request.background,
        request.padding_percent,
    )?);
    write_android_adaptive_files(root, &request.background)?;
    assets.push(file_asset(
        platform,
        &root.join("app/src/main/res/values/ic_launcher_background.xml"),
        0,
        0,
    )?);
    assets.push(file_asset(
        platform,
        &root.join("app/src/main/res/mipmap-anydpi-v26/ic_launcher.xml"),
        0,
        0,
    )?);
    assets.push(file_asset(
        platform,
        &root.join("app/src/main/res/mipmap-anydpi-v26/ic_launcher_round.xml"),
        0,
        0,
    )?);
    Ok(())
}

fn generate_ios_icons(
    tree: &usvg::Tree,
    root: &Path,
    platform: IconPlatform,
    request: &SvgIconSetRequest,
    assets: &mut Vec<IconAssetResult>,
) -> Result<(), String> {
    let app_icon_dir = root.join("Assets.xcassets/AppIcon.appiconset");
    std::fs::create_dir_all(&app_icon_dir)
        .map_err(|error| format!("创建 iOS 图标目录失败: {error}"))?;

    let specs = ios_icon_specs();
    let mut images = Vec::new();
    for spec in &specs {
        let filename = format!("AppIcon-{}@{}.png", spec.size.replace('.', "_"), spec.scale);
        let rel_path = format!("Assets.xcassets/AppIcon.appiconset/{filename}");
        assets.push(write_png_asset(
            tree,
            root,
            platform,
            &rel_path,
            spec.pixels,
            spec.pixels,
            &request.background,
            request.padding_percent,
        )?);
        images.push(serde_json::json!({
            "idiom": spec.idiom,
            "size": format!("{}x{}", spec.size, spec.size),
            "scale": spec.scale,
            "filename": filename,
        }));
    }

    let contents = serde_json::json!({
        "images": images,
        "info": {
            "author": "xcode",
            "version": 1
        }
    });
    let contents_path = app_icon_dir.join("Contents.json");
    std::fs::write(
        &contents_path,
        serde_json::to_string_pretty(&contents)
            .map_err(|error| format!("生成 Contents.json 失败: {error}"))?,
    )
    .map_err(|error| format!("写入 Contents.json 失败: {error}"))?;
    assets.push(file_asset(platform, &contents_path, 0, 0)?);
    Ok(())
}

fn generate_flutter_icons(
    tree: &usvg::Tree,
    root: &Path,
    request: &SvgIconSetRequest,
    assets: &mut Vec<IconAssetResult>,
) -> Result<(), String> {
    generate_android_icons(
        tree,
        &root.join("android"),
        IconPlatform::Flutter,
        request,
        assets,
    )?;
    generate_ios_icons(
        tree,
        &root.join("ios/Runner"),
        IconPlatform::Flutter,
        request,
        assets,
    )?;
    Ok(())
}

fn generate_electron_icons(
    tree: &usvg::Tree,
    root: &Path,
    platform: IconPlatform,
    request: &SvgIconSetRequest,
    assets: &mut Vec<IconAssetResult>,
) -> Result<(), String> {
    let icons_root = root.join("build/icons");
    let png_sizes = [16, 24, 32, 48, 64, 128, 256, 512, 1024];
    for size in png_sizes {
        assets.push(write_png_asset(
            tree,
            &icons_root,
            platform,
            &format!("png/{size}x{size}.png"),
            size,
            size,
            &request.background,
            request.padding_percent,
        )?);
    }
    assets.push(write_png_asset(
        tree,
        &icons_root,
        platform,
        "icon.png",
        1024,
        1024,
        &request.background,
        request.padding_percent,
    )?);
    assets.push(write_ico_asset(
        tree,
        &icons_root.join("icon.ico"),
        platform,
        request,
    )?);
    assets.push(write_icns_asset(
        tree,
        &icons_root.join("icon.icns"),
        platform,
        request,
    )?);
    Ok(())
}

fn generate_tauri_icons(
    tree: &usvg::Tree,
    root: &Path,
    platform: IconPlatform,
    request: &SvgIconSetRequest,
    assets: &mut Vec<IconAssetResult>,
) -> Result<(), String> {
    let icons_root = root.join("src-tauri/icons");
    let png_specs = [
        ("32x32.png", 32),
        ("128x128.png", 128),
        ("128x128@2x.png", 256),
        ("icon.png", 1024),
        ("Square30x30Logo.png", 30),
        ("Square44x44Logo.png", 44),
        ("Square71x71Logo.png", 71),
        ("Square89x89Logo.png", 89),
        ("Square107x107Logo.png", 107),
        ("Square142x142Logo.png", 142),
        ("Square150x150Logo.png", 150),
        ("Square284x284Logo.png", 284),
        ("Square310x310Logo.png", 310),
        ("StoreLogo.png", 50),
    ];

    for (filename, size) in png_specs {
        assets.push(write_png_asset(
            tree,
            &icons_root,
            platform,
            filename,
            size,
            size,
            &request.background,
            request.padding_percent,
        )?);
    }

    assets.push(write_ico_asset(
        tree,
        &icons_root.join("icon.ico"),
        platform,
        request,
    )?);
    assets.push(write_icns_asset(
        tree,
        &icons_root.join("icon.icns"),
        platform,
        request,
    )?);
    Ok(())
}

fn write_android_adaptive_files(root: &Path, background: &str) -> Result<(), String> {
    let values_dir = root.join("app/src/main/res/values");
    let adaptive_dir = root.join("app/src/main/res/mipmap-anydpi-v26");
    std::fs::create_dir_all(&values_dir)
        .map_err(|error| format!("创建 Android values 目录失败: {error}"))?;
    std::fs::create_dir_all(&adaptive_dir)
        .map_err(|error| format!("创建 Android adaptive icon 目录失败: {error}"))?;

    let color = normalize_rgb_hex(background);
    std::fs::write(
        values_dir.join("ic_launcher_background.xml"),
        format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
    <color name="ic_launcher_background">{color}</color>
</resources>
"#
        ),
    )
    .map_err(|error| format!("写入 Android 背景色失败: {error}"))?;

    let adaptive_xml = r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@color/ic_launcher_background" />
    <foreground android:drawable="@mipmap/ic_launcher_foreground" />
</adaptive-icon>
"#;
    std::fs::write(adaptive_dir.join("ic_launcher.xml"), adaptive_xml)
        .map_err(|error| format!("写入 Android adaptive icon 失败: {error}"))?;
    std::fs::write(adaptive_dir.join("ic_launcher_round.xml"), adaptive_xml)
        .map_err(|error| format!("写入 Android round adaptive icon 失败: {error}"))?;
    Ok(())
}

fn write_png_asset(
    tree: &usvg::Tree,
    root: &Path,
    platform: IconPlatform,
    relative_path: &str,
    width: u32,
    height: u32,
    background: &str,
    padding_percent: u8,
) -> Result<IconAssetResult, String> {
    let output_path = root.join(relative_path);
    let bytes = render_tree_png_bytes(tree, width, height, background, padding_percent)?;
    write_bytes(&output_path, &bytes)?;
    file_asset(platform, &output_path, width, height)
}

fn write_ico_asset(
    tree: &usvg::Tree,
    output_path: &Path,
    platform: IconPlatform,
    request: &SvgIconSetRequest,
) -> Result<IconAssetResult, String> {
    let sizes = [16, 24, 32, 48, 64, 128, 256];
    let mut images = Vec::new();
    for size in sizes {
        images.push((
            size,
            render_tree_png_bytes(
                tree,
                size,
                size,
                &request.background,
                request.padding_percent,
            )?,
        ));
    }

    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&(images.len() as u16).to_le_bytes());

    let mut offset = 6 + images.len() * 16;
    for (size, png) in &images {
        ico.push(if *size >= 256 { 0 } else { *size as u8 });
        ico.push(if *size >= 256 { 0 } else { *size as u8 });
        ico.push(0);
        ico.push(0);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&(png.len() as u32).to_le_bytes());
        ico.extend_from_slice(&(offset as u32).to_le_bytes());
        offset += png.len();
    }

    for (_, png) in images {
        ico.extend_from_slice(&png);
    }

    write_bytes(output_path, &ico)?;
    file_asset(platform, output_path, 0, 0)
}

fn write_icns_asset(
    tree: &usvg::Tree,
    output_path: &Path,
    platform: IconPlatform,
    request: &SvgIconSetRequest,
) -> Result<IconAssetResult, String> {
    let specs = [
        ("icp4", 16),
        ("icp5", 32),
        ("icp6", 64),
        ("ic07", 128),
        ("ic08", 256),
        ("ic09", 512),
        ("ic10", 1024),
    ];
    let mut chunks = Vec::new();
    let mut total_len = 8u32;

    for (kind, size) in specs {
        let png = render_tree_png_bytes(
            tree,
            size,
            size,
            &request.background,
            request.padding_percent,
        )?;
        let chunk_len = (8 + png.len()) as u32;
        total_len += chunk_len;
        chunks.push((kind, chunk_len, png));
    }

    let mut icns = Vec::new();
    icns.extend_from_slice(b"icns");
    icns.extend_from_slice(&total_len.to_be_bytes());
    for (kind, chunk_len, png) in chunks {
        icns.extend_from_slice(kind.as_bytes());
        icns.extend_from_slice(&chunk_len.to_be_bytes());
        icns.extend_from_slice(&png);
    }

    write_bytes(output_path, &icns)?;
    file_asset(platform, output_path, 0, 0)
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| format!("创建目录失败: {error}"))?;
    }
    std::fs::write(path, bytes).map_err(|error| format!("写入文件失败: {error}"))
}

fn file_asset(
    platform: IconPlatform,
    path: &Path,
    width: u32,
    height: u32,
) -> Result<IconAssetResult, String> {
    let output_bytes = std::fs::metadata(path)
        .map_err(|error| format!("读取生成文件信息失败: {error}"))?
        .len();
    Ok(IconAssetResult {
        platform,
        output_path: path.display().to_string(),
        width,
        height,
        output_bytes,
    })
}

fn render_tree_png_bytes(
    tree: &usvg::Tree,
    width: u32,
    height: u32,
    background: &str,
    padding_percent: u8,
) -> Result<Vec<u8>, String> {
    let pixmap = render_tree_pixmap(tree, width, height, background, true, padding_percent)?;
    let image = DynamicImage::ImageRgba8(rgba_image_from_pixmap(&pixmap)?);
    let mut cursor = Cursor::new(Vec::new());
    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|error| format!("编码 PNG 失败: {error}"))?;
    Ok(cursor.into_inner())
}

fn render_tree_pixmap(
    tree: &usvg::Tree,
    width: u32,
    height: u32,
    background: &str,
    keep_aspect_ratio: bool,
    padding_percent: u8,
) -> Result<Pixmap, String> {
    let clear_color =
        parse_color(background).unwrap_or_else(|| Color::from_rgba8(255, 255, 255, 0));
    render_tree_pixmap_with_color(
        tree,
        width,
        height,
        clear_color,
        keep_aspect_ratio,
        padding_percent,
    )
}

fn render_tree_pixmap_with_color(
    tree: &usvg::Tree,
    width: u32,
    height: u32,
    clear_color: Color,
    keep_aspect_ratio: bool,
    padding_percent: u8,
) -> Result<Pixmap, String> {
    let mut pixmap = Pixmap::new(width, height).ok_or_else(|| "无法创建目标画布".to_string())?;
    pixmap.fill(clear_color);

    let svg_size = tree.size();
    let source_width = svg_size.width();
    let source_height = svg_size.height();
    let padding_x = width as f32 * padding_percent as f32 / 100.0;
    let padding_y = height as f32 * padding_percent as f32 / 100.0;
    let target_width = (width as f32 - padding_x * 2.0).max(1.0);
    let target_height = (height as f32 - padding_y * 2.0).max(1.0);

    let transform = if keep_aspect_ratio {
        let scale = f32::min(target_width / source_width, target_height / source_height);
        let scaled_width = source_width * scale;
        let scaled_height = source_height * scale;
        let tx = (width as f32 - scaled_width) / 2.0;
        let ty = (height as f32 - scaled_height) / 2.0;
        Transform::from_scale(scale, scale).post_translate(tx, ty)
    } else {
        Transform::from_scale(target_width / source_width, target_height / source_height)
            .post_translate(padding_x, padding_y)
    };

    resvg::render(tree, transform, &mut pixmap.as_mut());
    Ok(pixmap)
}

fn rgba_image_from_pixmap(pixmap: &Pixmap) -> Result<RgbaImage, String> {
    ImageBuffer::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
        .ok_or_else(|| "无法从 SVG 渲染结果构建图像缓冲".to_string())
}

#[derive(Debug, Clone, Copy)]
struct IosIconSpec {
    idiom: &'static str,
    size: &'static str,
    scale: &'static str,
    pixels: u32,
}

fn ios_icon_specs() -> Vec<IosIconSpec> {
    vec![
        IosIconSpec {
            idiom: "iphone",
            size: "20",
            scale: "2x",
            pixels: 40,
        },
        IosIconSpec {
            idiom: "iphone",
            size: "20",
            scale: "3x",
            pixels: 60,
        },
        IosIconSpec {
            idiom: "iphone",
            size: "29",
            scale: "2x",
            pixels: 58,
        },
        IosIconSpec {
            idiom: "iphone",
            size: "29",
            scale: "3x",
            pixels: 87,
        },
        IosIconSpec {
            idiom: "iphone",
            size: "40",
            scale: "2x",
            pixels: 80,
        },
        IosIconSpec {
            idiom: "iphone",
            size: "40",
            scale: "3x",
            pixels: 120,
        },
        IosIconSpec {
            idiom: "iphone",
            size: "60",
            scale: "2x",
            pixels: 120,
        },
        IosIconSpec {
            idiom: "iphone",
            size: "60",
            scale: "3x",
            pixels: 180,
        },
        IosIconSpec {
            idiom: "ipad",
            size: "20",
            scale: "1x",
            pixels: 20,
        },
        IosIconSpec {
            idiom: "ipad",
            size: "20",
            scale: "2x",
            pixels: 40,
        },
        IosIconSpec {
            idiom: "ipad",
            size: "29",
            scale: "1x",
            pixels: 29,
        },
        IosIconSpec {
            idiom: "ipad",
            size: "29",
            scale: "2x",
            pixels: 58,
        },
        IosIconSpec {
            idiom: "ipad",
            size: "40",
            scale: "1x",
            pixels: 40,
        },
        IosIconSpec {
            idiom: "ipad",
            size: "40",
            scale: "2x",
            pixels: 80,
        },
        IosIconSpec {
            idiom: "ipad",
            size: "76",
            scale: "1x",
            pixels: 76,
        },
        IosIconSpec {
            idiom: "ipad",
            size: "76",
            scale: "2x",
            pixels: 152,
        },
        IosIconSpec {
            idiom: "ipad",
            size: "83.5",
            scale: "2x",
            pixels: 167,
        },
        IosIconSpec {
            idiom: "ios-marketing",
            size: "1024",
            scale: "1x",
            pixels: 1024,
        },
    ]
}

fn sanitize_file_stem(input: &str) -> String {
    let mut output = String::new();
    for character in input.trim().chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
        } else if character == '-' || character == '_' {
            output.push(character);
        } else if !output.ends_with('-') {
            output.push('-');
        }
    }
    let trimmed = output.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "app".to_string()
    } else {
        trimmed
    }
}

fn normalize_rgb_hex(input: &str) -> String {
    let raw = input.trim().trim_start_matches('#');
    if raw.len() >= 6
        && raw[0..6]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        format!("#{}", &raw[0..6])
    } else {
        "#ffffff".to_string()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
  <rect width="512" height="512" rx="96" fill="#21bd87"/>
  <path d="M160 268 236 344 368 168" fill="none" stroke="#fff" stroke-width="52" stroke-linecap="round" stroke-linejoin="round"/>
</svg>"##;

    #[test]
    fn exports_multi_platform_icon_set() {
        let root =
            std::env::temp_dir().join(format!("supertools-icon-set-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("create temp root");

        let result = export_svg_icon_set(SvgIconSetRequest {
            svg_code: SAMPLE_SVG.to_string(),
            output_dir: root.display().to_string(),
            app_name: "demo".to_string(),
            platforms: vec![
                IconPlatform::Android,
                IconPlatform::Ios,
                IconPlatform::Flutter,
                IconPlatform::Electron,
                IconPlatform::Tauri,
            ],
            background: "#21bd87".to_string(),
            padding_percent: 12,
        })
        .expect("export icon set");

        let output_root = root.join("demo_icons");
        assert!(result.generated_count > 50);
        assert!(output_root.join("source.svg").exists());
        assert!(output_root
            .join("android/app/src/main/res/mipmap-xxxhdpi/ic_launcher.png")
            .exists());
        assert!(output_root
            .join("ios/Assets.xcassets/AppIcon.appiconset/Contents.json")
            .exists());
        assert!(output_root
            .join("flutter/ios/Runner/Assets.xcassets/AppIcon.appiconset/Contents.json")
            .exists());
        assert!(output_root.join("electron/build/icons/icon.ico").exists());
        assert!(output_root.join("electron/build/icons/icon.icns").exists());
        assert!(output_root.join("tauri/src-tauri/icons/icon.png").exists());

        let _ = std::fs::remove_dir_all(&root);
    }
}
