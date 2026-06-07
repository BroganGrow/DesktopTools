import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { useEffect, useMemo, useState } from 'react'
import {
  cancelCompressJob,
  isTauriRuntime,
  openPath,
  resolveDroppedInput,
  scanPngInputs,
  startCompressJob,
} from '../lib/tauri'
import type {
  CompressJobRequest,
  CompressJobResult,
  CompressionOptions,
  CompressionPreset,
  CompressProgressEvent,
  InputSource,
  OutputFormat,
  ScanResult,
} from '../lib/types'
import { formatBytes, formatDuration, getErrorMessage } from '../lib/format'

const presetCopy: Record<CompressionPreset, string> = {
  balanced: '均衡',
  quality: '高质量',
  size: '更小体积',
}

const defaultOptions: CompressionOptions = {
  outputFormat: 'webp',
  preset: 'balanced',
  stripMetadata: true,
  keepTransparentRgb: true,
  skipLargerOutput: true,
  conflictPolicy: 'replace',
  outputSuffix: '_compressed',
  recursive: false,
}

export function PngCompressTool() {
  const [inputSource, setInputSource] = useState<InputSource | null>(null)
  const [scanResult, setScanResult] = useState<ScanResult | null>(null)
  const [options, setOptions] = useState<CompressionOptions>(defaultOptions)
  const [isBusy, setIsBusy] = useState(false)
  const [jobResult, setJobResult] = useState<CompressJobResult | null>(null)
  const [progressEvent, setProgressEvent] = useState<CompressProgressEvent | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [dragOver, setDragOver] = useState(false)
  const runtimeReady = isTauriRuntime()

  useEffect(() => {
    if (!runtimeReady) {
      return
    }

    let unlisten: (() => void) | undefined
    void listen<CompressProgressEvent>('compress://progress', (event) => {
      setProgressEvent(event.payload)
    }).then((dispose) => {
      unlisten = dispose
    })

    return () => {
      unlisten?.()
    }
  }, [runtimeReady])

  useEffect(() => {
    if (!runtimeReady) {
      return
    }

    let unlistenDrag: (() => void) | undefined
    void getCurrentWindow()
      .onDragDropEvent(async (event) => {
        if (event.payload.type === 'enter') {
          setDragOver(true)
          return
        }

        if (event.payload.type === 'leave') {
          setDragOver(false)
          return
        }

        if (event.payload.type === 'drop') {
          setDragOver(false)
          setError(null)
          try {
            const source = await resolveDroppedInput(event.payload.paths)
            await refreshScan(source)
          } catch (dropError) {
            setError(getErrorMessage(dropError))
          }
        }
      })
      .then((dispose) => {
        unlistenDrag = dispose
      })

    return () => {
      unlistenDrag?.()
    }
  }, [runtimeReady])

  const selectedSummary = useMemo(() => {
    if (!inputSource) {
      return '还没有选择输入源'
    }
    return inputSource.kind === 'folder' ? `已选择文件夹：${inputSource.path}` : `已选择 ${inputSource.paths.length} 个文件`
  }, [inputSource])

  async function handleSelectFiles() {
    setError(null)
    if (!runtimeReady) {
      setError('当前是浏览器预览模式。请使用 `npm run tauri:dev` 启动桌面应用。')
      return
    }

    const result = await open({
      multiple: true,
      directory: false,
      filters: [{ name: 'PNG 图片', extensions: ['png'] }],
      title: '选择要压缩的 PNG 文件',
    })

    if (!result || typeof result === 'string') {
      return
    }

    await refreshScan({ kind: 'files', paths: result })
  }

  async function handleSelectFolder() {
    setError(null)
    if (!runtimeReady) {
      setError('当前是浏览器预览模式。请使用 `npm run tauri:dev` 启动桌面应用。')
      return
    }

    const result = await open({
      multiple: false,
      directory: true,
      title: '选择包含 PNG 的文件夹',
    })

    if (!result || Array.isArray(result)) {
      return
    }

    await refreshScan({ kind: 'folder', path: result, recursive: options.recursive })
  }

  async function refreshScan(source: InputSource) {
    setInputSource(source)
    setJobResult(null)
    setProgressEvent(null)

    try {
      const nextScan = await scanPngInputs(source)
      setScanResult(nextScan)
      if (nextScan.supportedCount === 0) {
        setError('没有扫描到可处理的 PNG 文件。')
      }
    } catch (scanError) {
      setError(getErrorMessage(scanError))
    }
  }

  async function handleToggleRecursive(enabled: boolean) {
    setOptions((current) => ({ ...current, recursive: enabled }))
    if (inputSource?.kind === 'folder') {
      await refreshScan({ ...inputSource, recursive: enabled })
    }
  }

  async function handleStart() {
    if (!inputSource || !scanResult) {
      setError('请先选择输入文件。')
      return
    }
    if (scanResult.supportedCount === 0) {
      setError('当前没有可处理文件。')
      return
    }

    setError(null)
    setIsBusy(true)
    setJobResult(null)
    setProgressEvent(null)

    const request: CompressJobRequest = { input: inputSource, options }

    try {
      const result = await startCompressJob(request)
      setJobResult(result)
    } catch (jobError) {
      setError(getErrorMessage(jobError))
    } finally {
      setIsBusy(false)
    }
  }

  async function handleCancel() {
    if (!progressEvent?.jobId) {
      return
    }
    try {
      await cancelCompressJob(progressEvent.jobId)
    } catch (cancelError) {
      setError(getErrorMessage(cancelError))
    }
  }

  return (
    <div className="tool-stage">
      <section className="module-card">
        <div className="section-top">
          <div>
            <h4>快速设置</h4>
            <p>{selectedSummary}</p>
          </div>
        </div>
        {dragOver ? <p className="drop-hint">松开鼠标以导入：单文件 / 多文件 / 单个文件夹</p> : null}

        <div className="action-row png-action-grid">
          <button className="secondary" onClick={handleSelectFiles} type="button">选择文件（单个/多个）</button>
          <button className="secondary" onClick={handleSelectFolder} type="button">选择文件夹</button>
          <button className="primary" disabled={isBusy || !scanResult || scanResult.supportedCount === 0} onClick={handleStart} type="button">
            {isBusy ? '压缩中…' : '开始压缩'}
          </button>
          <button className="ghost" disabled={!isBusy || !progressEvent?.jobId} onClick={handleCancel} type="button">取消任务</button>
        </div>

        <div className="form-grid png-quick-grid">
          <label>
            <span>导出格式</span>
            <select
              value={options.outputFormat}
              onChange={(event) =>
                setOptions((current) => ({ ...current, outputFormat: event.target.value as OutputFormat }))
              }
            >
              <option value="webp">WebP（推荐）</option>
              <option value="avif">AVIF（更小体积）</option>
              <option value="png">PNG</option>
              <option value="jpeg">JPG / JPEG</option>
            </select>
          </label>
          <label>
            <span>预设</span>
            <select value={options.preset} onChange={(event) => setOptions((current) => ({ ...current, preset: event.target.value as CompressionPreset }))}>
              {Object.entries(presetCopy).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
            </select>
          </label>
          <label>
            <span>质量覆盖</span>
            <input type="number" min={1} max={100} placeholder="留空使用预设" value={options.quality ?? ''} onChange={(event) => setOptions((current) => ({ ...current, quality: event.target.value ? Number(event.target.value) : undefined }))} />
          </label>
          <label>
            <span>待处理</span>
            <input readOnly value={String(scanResult?.supportedCount ?? 0)} />
          </label>
          <label>
            <span>原始体积</span>
            <input readOnly value={formatBytes(scanResult?.totalBytes ?? 0)} />
          </label>
        </div>

        <div className="toggle-grid png-toggle-grid">
          <label className="toggle-panel">
            <input
              checked={options.recursive}
              onChange={(event) => void handleToggleRecursive(event.target.checked)}
              type="checkbox"
            />
            <div>
              <strong>递归扫描子目录</strong>
              <p>只在文件夹模式生效。</p>
            </div>
          </label>
          <label className="toggle-panel">
            <input
              checked={options.skipLargerOutput}
              onChange={(event) => setOptions((current) => ({ ...current, skipLargerOutput: event.target.checked }))}
              type="checkbox"
            />
            <div>
              <strong>只保留更小结果</strong>
              <p>默认开启。</p>
            </div>
          </label>
        </div>
      </section>

      <section className="module-card">
        <div className="section-top">
          <div>
            <h4>执行与结果</h4>
            <p>聚焦进度、节省体积和错误原因。</p>
          </div>
        </div>

        {progressEvent ? (
          <div className="progress-panel">
            <div className="progress-row">
              <strong>{progressEvent.stateLabel}</strong>
              <span>{progressEvent.completed}/{progressEvent.total}</span>
            </div>
            <div className="progress-track">
              <div className="progress-fill" style={{ width: `${progressEvent.total === 0 ? 0 : (progressEvent.completed / progressEvent.total) * 100}%` }} />
            </div>
            <p>{progressEvent.currentFile || '等待任务开始…'}</p>
            <small>累计节省：{formatBytes(progressEvent.savedBytes)}</small>
          </div>
        ) : null}

        {error ? <p className="error-banner">{error}</p> : null}

        {jobResult ? (
          <div className="result-stack">
            <div className="dashboard-grid">
              <article className="metric-box"><span>成功</span><strong>{jobResult.succeeded}</strong><small>已完成文件</small></article>
              <article className="metric-box"><span>失败</span><strong>{jobResult.failed}</strong><small>未成功项</small></article>
              <article className="metric-box"><span>节省体积</span><strong>{formatBytes(jobResult.savedBytes)}</strong><small>累计收益</small></article>
              <article className="metric-box"><span>耗时</span><strong>{formatDuration(jobResult.durationMs)}</strong><small>本次任务</small></article>
            </div>

            <div className="action-row">
              {jobResult.outputRoots.slice(0, 2).map((root) => (
                <button className="secondary" key={root} onClick={() => void openPath(root)} type="button">打开输出目录</button>
              ))}
            </div>

            <div className="result-list">
              {jobResult.items.slice(0, 10).map((item) => (
                <div className="result-row" key={`${item.sourcePath}-${item.outputPath}`}>
                  <code>{item.sourcePath}</code>
                  <span className={item.status === 'succeeded' ? 'ok' : 'bad'}>
                    {item.status === 'succeeded' ? '成功' : item.status === 'skipped' ? '跳过' : '失败'}
                  </span>
                  <span>{item.status === 'succeeded' ? `${formatBytes(item.sourceBytes)} → ${formatBytes(item.outputBytes ?? 0)}` : item.error ?? '-'}</span>
                </div>
              ))}
            </div>
          </div>
        ) : null}
      </section>
    </div>
  )
}
