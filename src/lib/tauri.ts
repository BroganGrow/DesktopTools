import { invoke } from '@tauri-apps/api/core'
import type {
  CompressJobRequest,
  CompressJobResult,
  InputSource,
  ScanResult,
  SvgExportRequest,
  SvgExportResult,
  SvgIconSetRequest,
  SvgIconSetResult,
  ImagesToPdfRequest,
  MergePdfsRequest,
  PdfOperationResult,
  PdfToImagesRequest,
  SplitPdfRequest,
  WatermarkPdfRequest,
} from './types'

declare global {
  interface Window {
    __TAURI_INTERNALS__?: unknown
  }
}

export function isTauriRuntime() {
  return typeof window !== 'undefined' && Boolean(window.__TAURI_INTERNALS__)
}

export async function scanPngInputs(input: InputSource) {
  return invoke<ScanResult>('scan_png_inputs', { input })
}

export async function resolveDroppedInput(paths: string[]) {
  return invoke<InputSource>('resolve_dropped_input', { paths })
}

export async function startCompressJob(request: CompressJobRequest) {
  return invoke<CompressJobResult>('start_compress_job', { request })
}

export async function cancelCompressJob(jobId: string) {
  return invoke<boolean>('cancel_compress_job', { jobId })
}

export async function openPath(path: string) {
  return invoke<void>('open_path', { path })
}

export async function exportSvgImage(request: SvgExportRequest) {
  return invoke<SvgExportResult>('export_svg_image', { request })
}

export async function exportSvgIconSet(request: SvgIconSetRequest) {
  return invoke<SvgIconSetResult>('export_svg_icon_set', { request })
}

export async function imagesToPdf(request: ImagesToPdfRequest) {
  return invoke<PdfOperationResult>('images_to_pdf', { request })
}

export async function pdfToImages(request: PdfToImagesRequest) {
  return invoke<PdfOperationResult>('pdf_to_images', { request })
}

export async function mergePdfs(request: MergePdfsRequest) {
  return invoke<PdfOperationResult>('merge_pdfs', { request })
}

export async function splitPdf(request: SplitPdfRequest) {
  return invoke<PdfOperationResult>('split_pdf', { request })
}

export async function watermarkPdf(request: WatermarkPdfRequest) {
  return invoke<PdfOperationResult>('watermark_pdf', { request })
}
