use crate::{
    app_state::{AppState, JobControl},
    compress::{run_job, scan_inputs, CompressJobRequest, InputSource, ScanResult},
    pdf_tools::{
        images_to_pdf_file, merge_pdfs_file, pdf_to_images_file, split_pdf_file,
        watermark_pdf_file, ImagesToPdfRequest, MergePdfsRequest, PdfOperationResult,
        PdfToImagesRequest, SplitPdfRequest, WatermarkPdfRequest,
    },
    svg::{
        export_svg_icon_set as export_svg_icon_set_impl, export_svg_image_file, SvgExportRequest,
        SvgExportResult, SvgIconSetRequest, SvgIconSetResult,
    },
};
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn scan_png_inputs(input: InputSource) -> Result<ScanResult, String> {
    scan_inputs(&input)
}

#[tauri::command]
pub fn resolve_dropped_input(paths: Vec<String>) -> Result<InputSource, String> {
    if paths.is_empty() {
        return Err("未检测到拖拽路径".to_string());
    }

    let mut files = Vec::new();
    let mut folders = Vec::new();

    for raw in paths {
        let path = std::path::PathBuf::from(&raw);
        let metadata = std::fs::metadata(&path)
            .map_err(|error| format!("读取拖拽路径失败: {raw} ({error})"))?;
        if metadata.is_dir() {
            folders.push(raw);
        } else if metadata.is_file() {
            files.push(raw);
        }
    }

    if folders.len() == 1 && files.is_empty() {
        return Ok(InputSource::Folder {
            path: folders.remove(0),
            recursive: false,
        });
    }

    if !files.is_empty() {
        return Ok(InputSource::Files { paths: files });
    }

    Err("未检测到可处理的文件或文件夹".to_string())
}

#[tauri::command]
pub async fn start_compress_job(
    app: AppHandle,
    state: State<'_, AppState>,
    request: CompressJobRequest,
) -> Result<crate::compress::CompressJobResult, String> {
    if state.current_job().is_some() {
        return Err("当前已有任务在运行，请先等待完成或取消。".to_string());
    }

    let job_id = format!(
        "job-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis()
    );

    let control = Arc::new(JobControl::new(job_id.clone()));
    state.set_current_job(Arc::clone(&control));

    let result = run_job(app, request, Arc::clone(&control)).await;
    state.clear_current_job(&job_id);
    result
}

#[tauri::command]
pub fn cancel_compress_job(job_id: String, state: State<'_, AppState>) -> Result<bool, String> {
    let Some(job) = state.current_job() else {
        return Ok(false);
    };

    if job.id != job_id {
        return Ok(false);
    }

    job.cancel();
    Ok(true)
}

#[tauri::command]
pub fn open_path(path: String) -> Result<(), String> {
    open::that_detached(path).map_err(|error| format!("打开路径失败: {error}"))
}

#[tauri::command]
pub fn export_svg_image(request: SvgExportRequest) -> Result<SvgExportResult, String> {
    export_svg_image_file(request)
}

#[tauri::command]
pub fn export_svg_icon_set(request: SvgIconSetRequest) -> Result<SvgIconSetResult, String> {
    export_svg_icon_set_impl(request)
}

#[tauri::command]
pub async fn images_to_pdf(
    app: AppHandle,
    request: ImagesToPdfRequest,
) -> Result<PdfOperationResult, String> {
    images_to_pdf_file(app, request).await
}

#[tauri::command]
pub async fn pdf_to_images(
    app: AppHandle,
    request: PdfToImagesRequest,
) -> Result<PdfOperationResult, String> {
    pdf_to_images_file(app, request).await
}

#[tauri::command]
pub async fn merge_pdfs(
    app: AppHandle,
    request: MergePdfsRequest,
) -> Result<PdfOperationResult, String> {
    merge_pdfs_file(app, request).await
}

#[tauri::command]
pub async fn split_pdf(
    app: AppHandle,
    request: SplitPdfRequest,
) -> Result<PdfOperationResult, String> {
    split_pdf_file(app, request).await
}

#[tauri::command]
pub async fn watermark_pdf(
    app: AppHandle,
    request: WatermarkPdfRequest,
) -> Result<PdfOperationResult, String> {
    watermark_pdf_file(app, request).await
}
