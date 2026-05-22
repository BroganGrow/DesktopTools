use crate::{
    app_state::{AppState, JobControl},
    compress::{run_job, scan_inputs, CompressJobRequest, InputSource, ScanResult},
    svg::{export_svg_image_file, SvgExportRequest, SvgExportResult},
};
use std::sync::Arc;
use tauri::{AppHandle, State};

#[tauri::command]
pub fn scan_png_inputs(input: InputSource) -> Result<ScanResult, String> {
    scan_inputs(&input)
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
