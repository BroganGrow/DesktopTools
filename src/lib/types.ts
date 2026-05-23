export type OutputFormat = 'webp' | 'png' | 'avif'
export type SvgExportFormat = 'png' | 'jpg' | 'jpeg'
export type IconPlatform = 'android' | 'ios' | 'flutter' | 'electron' | 'tauri'

export type CompressionPreset = 'balanced' | 'quality' | 'size'

export type ConflictPolicy = 'replace' | 'skip' | 'rename'

export type InputSource =
  | {
      kind: 'files'
      paths: string[]
    }
  | {
      kind: 'folder'
      path: string
    }

export type CompressionOptions = {
  outputFormat: OutputFormat
  preset: CompressionPreset
  quality?: number
  stripMetadata: boolean
  keepTransparentRgb: boolean
  conflictPolicy: ConflictPolicy
  outputSuffix: string
  recursive: boolean
}

export type CompressJobRequest = {
  input: InputSource
  options: CompressionOptions
}

export type SkippedItem = {
  path: string
  reason: string
}

export type ScanCandidate = {
  path: string
  bytes: number
}

export type ScanResult = {
  candidates: ScanCandidate[]
  skipped: SkippedItem[]
  supportedCount: number
  ignoredCount: number
  totalBytes: number
}

export type CompressItemResult = {
  sourcePath: string
  outputPath: string
  sourceBytes: number
  outputBytes?: number
  durationMs: number
  ratio?: number
  status: 'succeeded' | 'failed' | 'skipped'
  error?: string
}

export type CompressJobResult = {
  jobId: string
  succeeded: number
  failed: number
  skipped: number
  replaced: number
  durationMs: number
  savedBytes: number
  outputRoots: string[]
  items: CompressItemResult[]
}

export type CompressProgressEvent = {
  jobId: string
  currentFile?: string
  completed: number
  total: number
  savedBytes: number
  state: 'queued' | 'running' | 'cancelling' | 'completed'
  stateLabel: string
}

export type SvgExportRequest = {
  svgCode: string
  width: number
  height: number
  format: SvgExportFormat
  quality: number
  keepAspectRatio: boolean
  background: string
  paddingPercent: number
  outputPath: string
}

export type SvgExportResult = {
  outputPath: string
  outputDir: string
  width: number
  height: number
  format: SvgExportFormat
  outputBytes: number
}

export type SvgIconSetRequest = {
  svgCode: string
  outputDir: string
  appName: string
  platforms: IconPlatform[]
  background: string
  paddingPercent: number
}

export type IconAssetResult = {
  platform: IconPlatform
  outputPath: string
  width: number
  height: number
  outputBytes: number
}

export type SvgIconSetResult = {
  outputDir: string
  generatedCount: number
  totalBytes: number
  platforms: IconPlatform[]
  assets: IconAssetResult[]
}
