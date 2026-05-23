use crate::{app_state::JobControl, png::inspect_png};
use futures_util::{stream::FuturesUnordered, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
    time::Instant,
};
use tauri::{AppHandle, Emitter};
use tauri_plugin_shell::ShellExt;
use tokio::sync::Semaphore;

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum InputSource {
    Files { paths: Vec<String> },
    Folder { path: String },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum OutputFormat {
    Webp,
    Png,
    Avif,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CompressionPreset {
    Balanced,
    Quality,
    Size,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ConflictPolicy {
    Replace,
    Skip,
    Rename,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressionOptions {
    pub output_format: OutputFormat,
    pub preset: CompressionPreset,
    pub quality: Option<u8>,
    pub strip_metadata: bool,
    pub keep_transparent_rgb: bool,
    pub conflict_policy: ConflictPolicy,
    pub output_suffix: String,
    pub recursive: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressJobRequest {
    pub input: InputSource,
    pub options: CompressionOptions,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanCandidate {
    pub path: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedItem {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub candidates: Vec<ScanCandidate>,
    pub skipped: Vec<SkippedItem>,
    pub supported_count: usize,
    pub ignored_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressItemResult {
    pub source_path: String,
    pub output_path: String,
    pub source_bytes: u64,
    pub output_bytes: Option<u64>,
    pub duration_ms: u128,
    pub ratio: Option<f64>,
    pub status: ItemStatus,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ItemStatus {
    Succeeded,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressJobResult {
    pub job_id: String,
    pub succeeded: usize,
    pub failed: usize,
    pub skipped: usize,
    pub replaced: usize,
    pub duration_ms: u128,
    pub saved_bytes: u64,
    pub output_roots: Vec<String>,
    pub items: Vec<CompressItemResult>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompressProgressEvent {
    pub job_id: String,
    pub current_file: Option<String>,
    pub completed: usize,
    pub total: usize,
    pub saved_bytes: u64,
    pub state: ProgressState,
    pub state_label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProgressState {
    Queued,
    Running,
    Cancelling,
    Completed,
}

impl ProgressState {
    fn label(&self) -> String {
        match self {
            ProgressState::Queued => "任务已排队".to_string(),
            ProgressState::Running => "正在压缩".to_string(),
            ProgressState::Cancelling => "正在取消".to_string(),
            ProgressState::Completed => "任务完成".to_string(),
        }
    }
}

pub fn scan_inputs(input: &InputSource) -> Result<ScanResult, String> {
    let mut skipped = Vec::new();
    let mut candidates = Vec::new();
    let mut ignored_count = 0usize;

    let paths = resolve_input_paths(input)?;
    for path in paths {
        if !is_png_path(&path) {
            ignored_count += 1;
            skipped.push(SkippedItem {
                path: path.display().to_string(),
                reason: "不是 PNG 文件".to_string(),
            });
            continue;
        }

        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                skipped.push(SkippedItem {
                    path: path.display().to_string(),
                    reason: format!("读取文件信息失败: {error}"),
                });
                continue;
            }
        };

        match inspect_png(&path) {
            Ok(inspection) if inspection.is_apng => skipped.push(SkippedItem {
                path: path.display().to_string(),
                reason: "APNG 暂不支持".to_string(),
            }),
            Ok(_) => candidates.push(ScanCandidate {
                path: path.display().to_string(),
                bytes: metadata.len(),
            }),
            Err(error) => skipped.push(SkippedItem {
                path: path.display().to_string(),
                reason: error,
            }),
        }
    }

    let total_bytes = candidates.iter().map(|item| item.bytes).sum();
    let supported_count = candidates.len();

    Ok(ScanResult {
        candidates,
        skipped,
        supported_count,
        ignored_count,
        total_bytes,
    })
}

pub async fn run_job(
    app: AppHandle,
    request: CompressJobRequest,
    control: Arc<JobControl>,
) -> Result<CompressJobResult, String> {
    if request.options.output_format != OutputFormat::Webp {
        return Err("v1 仅支持输出 WebP。".to_string());
    }

    let scan = scan_inputs(&request.input)?;
    if scan.supported_count == 0 {
        return Err("没有可处理的 PNG 文件。".to_string());
    }

    let total = scan.supported_count;
    emit_progress(&app, &control.id, None, 0, total, 0, ProgressState::Queued)?;

    let job_started = Instant::now();
    let limit = num_cpus::get_physical().max(1).min(4);
    let semaphore = Arc::new(Semaphore::new(limit));
    let mut pending = FuturesUnordered::new();

    for candidate in scan.candidates {
        let permit = Arc::clone(&semaphore);
        let app_handle = app.clone();
        let options = request.options.clone();
        let input = request.input.clone();
        let control = Arc::clone(&control);
        pending.push(tokio::spawn(async move {
            let _permit = permit
                .acquire_owned()
                .await
                .map_err(|error| error.to_string())?;
            if control.is_cancelled() {
                return Ok(build_cancelled_result(candidate.path, candidate.bytes));
            }

            compress_single(
                &app_handle,
                &input,
                candidate.path,
                candidate.bytes,
                &options,
            )
            .await
        }));
    }

    let mut completed = 0usize;
    let mut saved_bytes = 0u64;
    let mut succeeded = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;
    let mut replaced = 0usize;
    let mut output_roots = BTreeSet::new();
    let mut items = Vec::new();

    while let Some(result) = pending.next().await {
        let item = result.map_err(|error| error.to_string())??;
        completed += 1;

        match item.status {
            ItemStatus::Succeeded => {
                succeeded += 1;
                if let Some(output_bytes) = item.output_bytes {
                    saved_bytes += item.source_bytes.saturating_sub(output_bytes);
                }
            }
            ItemStatus::Failed => failed += 1,
            ItemStatus::Skipped => skipped += 1,
        }

        if item.error.as_deref() == Some("__replaced__") {
            replaced += 1;
        }

        if let Some(parent) = Path::new(&item.output_path).parent() {
            output_roots.insert(parent.display().to_string());
        }

        emit_progress(
            &app,
            &control.id,
            Some(item.source_path.clone()),
            completed,
            total,
            saved_bytes,
            if control.is_cancelled() {
                ProgressState::Cancelling
            } else {
                ProgressState::Running
            },
        )?;

        items.push(item);
    }

    emit_progress(
        &app,
        &control.id,
        None,
        completed,
        total,
        saved_bytes,
        ProgressState::Completed,
    )?;

    Ok(CompressJobResult {
        job_id: control.id.clone(),
        succeeded,
        failed,
        skipped,
        replaced,
        duration_ms: job_started.elapsed().as_millis(),
        saved_bytes,
        output_roots: output_roots.into_iter().collect(),
        items,
    })
}

fn emit_progress(
    app: &AppHandle,
    job_id: &str,
    current_file: Option<String>,
    completed: usize,
    total: usize,
    saved_bytes: u64,
    state: ProgressState,
) -> Result<(), String> {
    let payload = CompressProgressEvent {
        job_id: job_id.to_string(),
        current_file,
        completed,
        total,
        saved_bytes,
        state: state.clone(),
        state_label: state.label(),
    };

    app.emit("compress://progress", payload)
        .map_err(|error| format!("发送进度事件失败: {error}"))
}

async fn compress_single(
    app: &AppHandle,
    input: &InputSource,
    source_path: String,
    source_bytes: u64,
    options: &CompressionOptions,
) -> Result<CompressItemResult, String> {
    let started = Instant::now();
    let source = PathBuf::from(&source_path);
    let output_path = build_output_path(input, &source, &options.output_suffix)?;
    let output_dir = output_path
        .parent()
        .ok_or_else(|| "无法确定输出目录".to_string())?;

    fs::create_dir_all(output_dir).map_err(|error| format!("创建输出目录失败: {error}"))?;

    let mut replaced_existing = false;
    match options.conflict_policy {
        ConflictPolicy::Replace => {
            if output_path.exists() {
                replaced_existing = true;
                fs::remove_file(&output_path)
                    .map_err(|error| format!("覆盖旧文件失败: {error}"))?;
            }
        }
        ConflictPolicy::Skip if output_path.exists() => {
            return Ok(CompressItemResult {
                source_path,
                output_path: output_path.display().to_string(),
                source_bytes,
                output_bytes: None,
                duration_ms: started.elapsed().as_millis(),
                ratio: None,
                status: ItemStatus::Skipped,
                error: Some("输出文件已存在".to_string()),
            });
        }
        ConflictPolicy::Rename if output_path.exists() => {}
        _ => {}
    }

    let resolved_output = if matches!(options.conflict_policy, ConflictPolicy::Rename) {
        uniquify_output_path(output_path)
    } else {
        output_path
    };

    let mut args = preset_args(options);
    args.push(source_path.clone());
    args.push("-o".to_string());
    args.push(resolved_output.display().to_string());

    let command = app
        .shell()
        .sidecar("binaries/cwebp")
        .map_err(|error| format!("未找到 cwebp sidecar，请先准备平台二进制文件: {error}"))?;

    let output = command
        .args(args)
        .output()
        .await
        .map_err(|error| format!("执行 cwebp 失败: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Ok(CompressItemResult {
            source_path,
            output_path: resolved_output.display().to_string(),
            source_bytes,
            output_bytes: None,
            duration_ms: started.elapsed().as_millis(),
            ratio: None,
            status: ItemStatus::Failed,
            error: Some(if stderr.is_empty() {
                "cwebp 返回失败状态".to_string()
            } else {
                stderr
            }),
        });
    }

    let output_bytes = fs::metadata(&resolved_output)
        .map_err(|error| format!("读取输出文件失败: {error}"))?
        .len();

    Ok(CompressItemResult {
        source_path,
        output_path: resolved_output.display().to_string(),
        source_bytes,
        output_bytes: Some(output_bytes),
        duration_ms: started.elapsed().as_millis(),
        ratio: Some(output_bytes as f64 / source_bytes as f64),
        status: ItemStatus::Succeeded,
        error: replaced_existing.then(|| "__replaced__".to_string()),
    })
}

fn preset_args(options: &CompressionOptions) -> Vec<String> {
    let mut args = match options.preset {
        CompressionPreset::Balanced => vec![
            "-q".to_string(),
            "82".to_string(),
            "-m".to_string(),
            "6".to_string(),
            "-mt".to_string(),
            "-af".to_string(),
            "-sharp_yuv".to_string(),
            "-alpha_q".to_string(),
            "100".to_string(),
        ],
        CompressionPreset::Quality => vec![
            "-q".to_string(),
            "90".to_string(),
            "-m".to_string(),
            "6".to_string(),
            "-mt".to_string(),
            "-af".to_string(),
            "-sharp_yuv".to_string(),
            "-alpha_q".to_string(),
            "100".to_string(),
        ],
        CompressionPreset::Size => vec![
            "-q".to_string(),
            "75".to_string(),
            "-m".to_string(),
            "6".to_string(),
            "-mt".to_string(),
            "-af".to_string(),
            "-sharp_yuv".to_string(),
            "-alpha_q".to_string(),
            "90".to_string(),
        ],
    };

    if let Some(quality) = options.quality {
        args.splice(0..2, ["-q".to_string(), quality.clamp(1, 100).to_string()]);
    }

    if options.keep_transparent_rgb {
        args.push("-exact".to_string());
    }

    args.push("-metadata".to_string());
    args.push(if options.strip_metadata {
        "none".to_string()
    } else {
        "all".to_string()
    });

    args
}

fn build_output_path(input: &InputSource, source: &Path, suffix: &str) -> Result<PathBuf, String> {
    let stem = source
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "无法解析源文件名".to_string())?;

    let file_name = format!("{stem}{suffix}.webp");

    match input {
        InputSource::Folder { path } => {
            let folder = PathBuf::from(path);
            let parent = folder
                .parent()
                .ok_or_else(|| "文件夹缺少父目录".to_string())?;
            let folder_name = folder
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| "无法解析文件夹名".to_string())?;

            Ok(parent
                .join(format!("{folder_name}_compressed"))
                .join(file_name))
        }
        InputSource::Files { .. } => {
            let parent = source
                .parent()
                .ok_or_else(|| "源文件缺少父目录".to_string())?;
            let parent_name = parent
                .file_name()
                .and_then(|value| value.to_str())
                .ok_or_else(|| "无法解析源目录名".to_string())?;
            let parent_of_parent = parent
                .parent()
                .ok_or_else(|| "源目录缺少上级目录".to_string())?;

            Ok(parent_of_parent
                .join(format!("{parent_name}_compressed"))
                .join(file_name))
        }
    }
}

fn resolve_input_paths(input: &InputSource) -> Result<Vec<PathBuf>, String> {
    match input {
        InputSource::Files { paths } => Ok(paths.iter().map(PathBuf::from).collect()),
        InputSource::Folder { path } => {
            let entries = fs::read_dir(path).map_err(|error| format!("读取目录失败: {error}"))?;
            let mut files = Vec::new();
            for entry in entries {
                let entry = entry.map_err(|error| format!("遍历目录失败: {error}"))?;
                let entry_path = entry.path();
                if entry_path.is_file() {
                    files.push(entry_path);
                }
            }
            Ok(files)
        }
    }
}

fn is_png_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.eq_ignore_ascii_case("png"))
        .unwrap_or(false)
}

fn uniquify_output_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }

    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("output");
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("webp");
    let parent = path.parent().map(PathBuf::from).unwrap_or_default();

    for index in 1..=999 {
        let candidate = parent.join(format!("{stem}_{index}.{extension}"));
        if !candidate.exists() {
            return candidate;
        }
    }

    parent.join(format!("{stem}_overflow.{extension}"))
}

fn build_cancelled_result(source_path: String, source_bytes: u64) -> CompressItemResult {
    CompressItemResult {
        source_path,
        output_path: String::new(),
        source_bytes,
        output_bytes: None,
        duration_ms: 0,
        ratio: None,
        status: ItemStatus::Skipped,
        error: Some("任务已取消".to_string()),
    }
}
