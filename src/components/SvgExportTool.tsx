import { save } from '@tauri-apps/plugin-dialog'
import { useMemo, useState } from 'react'
import type { ChangeEvent } from 'react'
import { exportSvgImage, isTauriRuntime, openPath } from '../lib/tauri'
import type { SvgExportFormat, SvgExportRequest, SvgExportResult } from '../lib/types'
import { formatBytes, getErrorMessage } from '../lib/format'

const svgFormatCopy: Record<SvgExportFormat, string> = {
  png: 'PNG',
  jpg: 'JPG',
  jpeg: 'JPEG',
}

const defaultSvgMarkup = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 960 540">
  <defs>
    <linearGradient id="bg" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#10131a" />
      <stop offset="100%" stop-color="#204e8d" />
    </linearGradient>
  </defs>
  <rect width="960" height="540" fill="url(#bg)" rx="36" />
  <circle cx="170" cy="160" r="84" fill="#f0a458" opacity="0.9" />
  <text x="90" y="320" fill="#f5efe2" font-size="72" font-family="Segoe UI, Microsoft YaHei, sans-serif" font-weight="700">
    SuperTools
  </text>
  <text x="92" y="382" fill="#d5d0c4" font-size="28" font-family="Segoe UI, Microsoft YaHei, sans-serif">
    SVG 代码导出 PNG / JPG
  </text>
</svg>`

export function SvgExportTool() {
  const runtimeReady = isTauriRuntime()
  const [svgCode, setSvgCode] = useState(defaultSvgMarkup)
  const [width, setWidth] = useState(1200)
  const [height, setHeight] = useState(675)
  const [format, setFormat] = useState<SvgExportFormat>('png')
  const [quality, setQuality] = useState(92)
  const [keepAspect, setKeepAspect] = useState(true)
  const [background, setBackground] = useState('#ffffff')
  const [outputPath, setOutputPath] = useState('')
  const [isBusy, setIsBusy] = useState(false)
  const [result, setResult] = useState<SvgExportResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const previewUrl = useMemo(() => `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svgCode)}`, [svgCode])

  async function handleChooseOutput() {
    if (!runtimeReady) {
      setError('当前是浏览器预览模式。请使用 `npm run tauri:dev` 启动桌面应用。')
      return ''
    }
    const selected = await save({
      title: '选择导出图片位置',
      defaultPath: `supertools-export.${format === 'jpeg' ? 'jpg' : format}`,
      filters: [{ name: '图片文件', extensions: format === 'png' ? ['png'] : ['jpg', 'jpeg'] }],
    })
    if (!selected) {
      return ''
    }
    setOutputPath(selected)
    return selected
  }

  async function handleExport() {
    if (!svgCode.trim()) {
      setError('请先输入 SVG 代码。')
      return
    }
    let nextOutputPath = outputPath
    if (!nextOutputPath) {
      nextOutputPath = await handleChooseOutput()
    }
    if (!nextOutputPath) {
      return
    }

    setError(null)
    setIsBusy(true)
    setResult(null)
    const request: SvgExportRequest = {
      svgCode,
      width,
      height,
      format,
      quality,
      keepAspectRatio: keepAspect,
      background,
      outputPath: nextOutputPath,
    }

    try {
      const exported = await exportSvgImage(request)
      setResult(exported)
    } catch (exportError) {
      setError(getErrorMessage(exportError))
    } finally {
      setIsBusy(false)
    }
  }

  async function handleImportFile(event: ChangeEvent<HTMLInputElement>) {
    const file = event.target.files?.[0]
    if (!file) {
      return
    }
    try {
      setSvgCode(await file.text())
      setError(null)
    } catch (readError) {
      setError(getErrorMessage(readError))
    } finally {
      event.target.value = ''
    }
  }

  return (
    <div className="tool-stage">
      <section className="module-banner accent-vector">
        <div>
          <p className="eyebrow">SVG Export</p>
          <h3>先看，再导</h3>
          <p>把 SVG 工具做成偏创作工作台，而不是一串枯燥表单。你应该先判断画面，再决定是否导出。</p>
        </div>
        <div className="module-banner-meta">
          <span>实时预览</span>
          <span>目标画布</span>
          <span>resvg</span>
        </div>
      </section>

      <section className="module-card">
        <div className="section-top">
          <div>
            <h4>SVG 代码输入</h4>
            <p>支持粘贴代码或导入本地 SVG 文件。模块布局上优先把“内容”和“结果”靠近放置。</p>
          </div>
          <div className="action-row">
            <label className="secondary file-trigger">
              导入 SVG 文件
              <input accept=".svg,image/svg+xml" onChange={handleImportFile} type="file" />
            </label>
          </div>
        </div>

        <textarea className="code-editor" onChange={(event) => setSvgCode(event.target.value)} spellCheck={false} value={svgCode} />

        <div className="preview-grid">
          <article className="preview-card">
            <div className="preview-head">
              <strong>实时预览</strong>
              <span>原始 SVG 渲染</span>
            </div>
            <div className="preview-stage preview-stage-transparent">
              {previewUrl ? <img alt="SVG 实时预览" className="preview-image preview-image-contain" src={previewUrl} /> : <p className="preview-empty">等待 SVG 内容…</p>}
            </div>
          </article>

          <article className="preview-card">
            <div className="preview-head">
              <strong>目标画布预览</strong>
              <span>{width} × {height} · {keepAspect ? '保持比例' : '铺满画布'}</span>
            </div>
            <div className={`preview-stage ${format === 'png' ? 'preview-stage-transparent' : ''}`} style={{ background: format === 'png' ? undefined : background }}>
              {previewUrl ? (
                <div className={`canvas-frame ${keepAspect ? 'is-contain' : 'is-cover'}`} style={{ aspectRatio: `${width} / ${height}` }}>
                  <img alt="目标画布预览" className={`preview-image ${keepAspect ? 'preview-image-contain' : 'preview-image-cover'}`} src={previewUrl} />
                </div>
              ) : (
                <p className="preview-empty">等待 SVG 内容…</p>
              )}
            </div>
          </article>
        </div>
      </section>

      <section className="module-card">
        <div className="section-top">
          <div>
            <h4>导出参数</h4>
            <p>这里保留必要控制，但不抢视觉注意力。工作台的主角应该是预览区。</p>
          </div>
        </div>

        <div className="form-grid">
          <label>
            <span>宽度</span>
            <input min={1} onChange={(event) => setWidth(Number(event.target.value) || 1)} type="number" value={width} />
          </label>
          <label>
            <span>高度</span>
            <input min={1} onChange={(event) => setHeight(Number(event.target.value) || 1)} type="number" value={height} />
          </label>
          <label>
            <span>格式</span>
            <select value={format} onChange={(event) => setFormat(event.target.value as SvgExportFormat)}>
              {Object.entries(svgFormatCopy).map(([value, label]) => <option key={value} value={value}>{label}</option>)}
            </select>
          </label>
          <label>
            <span>质量</span>
            <input disabled={format === 'png'} max={100} min={1} onChange={(event) => setQuality(Number(event.target.value) || 1)} type="number" value={quality} />
          </label>
        </div>

        <div className="toggle-grid">
          <label className="toggle-panel">
            <input checked={keepAspect} onChange={(event) => setKeepAspect(event.target.checked)} type="checkbox" />
            <div>
              <strong>保持原始比例</strong>
              <p>关闭后会按目标画布拉伸。</p>
            </div>
          </label>
          <label className="toggle-panel align-end">
            <div>
              <strong>背景色</strong>
              <p>JPG 会直接铺底，PNG 可保留透明。</p>
            </div>
            <input className="color-input" onChange={(event) => setBackground(event.target.value)} type="color" value={background} />
          </label>
        </div>
      </section>

      <section className="module-card">
        <div className="section-top">
          <div>
            <h4>导出与结果</h4>
            <p>适合频繁试尺寸和试格式的短流程，不需要来回切屏和找文件夹。</p>
          </div>
          <div className="action-row">
            <button className="secondary" onClick={() => void handleChooseOutput()} type="button">选择导出位置</button>
            <button className="primary" disabled={isBusy} onClick={() => void handleExport()} type="button">{isBusy ? '导出中…' : '导出图片'}</button>
          </div>
        </div>

        <div className="dashboard-grid dashboard-grid-two">
          <article className="metric-box">
            <span>目标尺寸</span>
            <strong>{width} × {height}</strong>
            <small>导出格式：{svgFormatCopy[format]}</small>
          </article>
          <article className="metric-box">
            <span>输出路径</span>
            <strong className="metric-path">{outputPath || '尚未选择'}</strong>
            <small>{keepAspect ? '保持比例并居中' : '按目标画布铺满'}</small>
          </article>
        </div>

        {error ? <p className="error-banner">{error}</p> : null}

        {result ? (
          <div className="result-stack">
            <div className="dashboard-grid dashboard-grid-three">
              <article className="metric-box"><span>输出文件</span><strong>{result.format.toUpperCase()}</strong><small>已完成导出</small></article>
              <article className="metric-box"><span>生成尺寸</span><strong>{result.width} × {result.height}</strong><small>目标画布</small></article>
              <article className="metric-box"><span>文件大小</span><strong>{formatBytes(result.outputBytes)}</strong><small>导出结果</small></article>
            </div>
            <div className="action-row">
              <button className="secondary" onClick={() => void openPath(result.outputPath)} type="button">打开导出文件</button>
              <button className="secondary" onClick={() => void openPath(result.outputDir)} type="button">打开所在目录</button>
            </div>
          </div>
        ) : null}
      </section>
    </div>
  )
}
