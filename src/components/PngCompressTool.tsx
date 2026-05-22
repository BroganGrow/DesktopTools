import { open } from '@tauri-apps/plugin-dialog'
import { listen } from '@tauri-apps/api/event'
import { useEffect, useMemo, useState } from 'react'
import {
  cancelCompressJob,
  isTauriRuntime,
  openPath,
  scanPngInputs,
  startCompressJob,
} from '../lib/tauri'
import type {
  CompressJobRequest,
  CompressJobResult,
  CompressionOptions,
  CompressionPreset,
  CompressProgressEvent,
  ConflictPolicy,
  InputSource,
  ScanResult,
} from '../lib/types'
import { formatBytes, formatDuration, getErrorMessage } from '../lib/format'

const presetCopy: Record<CompressionPreset, string> = {
  balanced: '均衡',
  quality: '高质量',
  size: '更小体积',
}

const conflictCopy: Record<ConflictPolicy, string> = {
  replace: '覆盖已生成文件',
  skip: '跳过已存在文件',
  rename: '自动改名',
}

const defaultOptions: CompressionOptions = {
  outputFormat: 'webp',
  preset: 'balanced',
  stripMetadata: true,
  keepTransparentRgb: true,
  conflictPolicy: 'replace',
  outputSuffix: '_compressed',
  recursive: false,
}

export function PngCompressTool() {
  const [selectionMode, setSelectionMode] = useState<'files' | 'folder'>('files')
  const [inputSource, setInputSource] = useState<InputSource | null>(null)
  const [scanResult, setScanResult] = useState<ScanResult | null>(null)
  const [options, setOptions] = useState<CompressionOptions>(defaultOptions)
  const [isBusy, setIsBusy] = useState(false)
  const [jobResult, setJobResult] = useState<CompressJobResult | null>(null)
  const [progressEvent, setProgressEvent] = useState<CompressProgressEvent | null>(null)
  const [error, setError] = useState<string | null>(null)
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

    setSelectionMode('files')
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

    setSelectionMode('folder')
    await refreshScan({ kind: 'folder', path: result })
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

  const summaryBlocks = [
    { label: '待处理', value: scanResult?.supportedCount ?? 0, hint: '已通过校验的 PNG' },
    { label: '跳过', value: scanResult?.skipped.length ?? 0, hint: '非 PNG / APNG / 损坏文件' },
    { label: '原始体积', value: formatBytes(scanResult?.totalBytes ?? 0), hint: selectedSummary },
  ]

  return (
    <div className="tool-stage">
      <section className="module-banner accent-image">
        <div>
          <p className="eyebrow">PNG Compression</p>
          <h3>批量压缩，不碰原图</h3>
          <p>更适合高频生产使用的批处理型模块，输出目录和冲突策略都明确可控。</p>
        </div>
        <div className="module-banner-meta">
          <span>批量</span>
          <span>事件进度</span>
          <span>cwebp</span>
        </div>
      </section>

      <section className="module-card">
        <div className="section-top">
          <div>
            <h4>输入源</h4>
            <p>支持多选文件，或选择一个只扫描当前层级的文件夹。</p>
          </div>
          <div className="segmented">
            <button className={selectionMode === 'files' ? 'active' : ''} onClick={() => setSelectionMode('files')} type="button">
              多文件
            </button>
            <button className={selectionMode === 'folder' ? 'active' : ''} onClick={() => setSelectionMode('folder')} type="button">
              文件夹
            </button>
          </div>
        </div>

        <div className="action-row">
          <button className="secondary" onClick={handleSelectFiles} type="button">选择 PNG 文件</button>
          <button className="secondary" onClick={handleSelectFolder} type="button">选择文件夹</button>
        </div>

        <div className="dashboard-grid">
          {summaryBlocks.map((item) => (
            <article className="metric-box" key={item.label}>
              <span>{item.label}</span>
              <strong>{item.value}</strong>
              <small>{item.hint}</small>
            </article>
          ))}
        </div>
      </section>

      <section className="module-card">
        <div className="section-top">
          <div>
            <h4>压缩参数</h4>
            <p>主参数保持简洁，避免把常用模块做成一张密密麻麻的专业面板。</p>
          </div>
        </div>

        <div className="form-grid">
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
            <span>冲突策略</span>
            <select value={options.conflictPolicy} onChange={(event) => setOptions((current) => ({ ...current, conflictPolicy: event.target.value as ConflictPolicy }))}>
              {Object.entries(conflictCopy).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
            </select>
          </label>
          <label>
            <span>文件后缀</span>
            <input type="text" value={options.outputSuffix} onChange={(event) => setOptions((current) => ({ ...current, outputSuffix: event.target.value || '_compressed' }))} />
          </label>
        </div>

        <div className="toggle-grid">
          <label className="toggle-panel">
            <input checked={options.stripMetadata} onChange={(event) => setOptions((current) => ({ ...current, stripMetadata: event.target.checked }))} type="checkbox" />
            <div>
              <strong>移除元数据</strong>
              <p>默认开启，优先换取体积收益。</p>
            </div>
          </label>
          <label className="toggle-panel">
            <input checked={options.keepTransparentRgb} onChange={(event) => setOptions((current) => ({ ...current, keepTransparentRgb: event.target.checked }))} type="checkbox" />
            <div>
              <strong>保留透明区 RGB</strong>
              <p>减少透明边缘出现脏边。</p>
            </div>
          </label>
        </div>
      </section>

      <section className="module-card">
        <div className="section-top">
          <div>
            <h4>执行与结果</h4>
            <p>保持单文件失败不阻断整批流程，输出结果聚焦体积和错误原因。</p>
          </div>
          <div className="action-row">
            <button className="primary" disabled={isBusy || !scanResult || scanResult.supportedCount === 0} onClick={handleStart} type="button">
              {isBusy ? '压缩中…' : '开始压缩'}
            </button>
            <button className="ghost" disabled={!isBusy || !progressEvent?.jobId} onClick={handleCancel} type="button">取消任务</button>
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
