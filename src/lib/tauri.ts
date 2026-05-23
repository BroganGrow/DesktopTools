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
