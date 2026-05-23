import { open, save } from '@tauri-apps/plugin-dialog'
import { useMemo, useState } from 'react'
import type { ChangeEvent } from 'react'
import { exportSvgIconSet, exportSvgImage, isTauriRuntime, openPath } from '../lib/tauri'
import type {
  IconPlatform,
  SvgExportFormat,
  SvgExportRequest,
  SvgExportResult,
  SvgIconSetRequest,
  SvgIconSetResult,
} from '../lib/types'
import { formatBytes, getErrorMessage } from '../lib/format'

const defaultSvgMarkup = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 512 512">
  <rect width="512" height="512" rx="112" fill="#21bd87"/>
  <path d="M160 268 236 344 368 168" fill="none" stroke="#fff" stroke-width="52" stroke-linecap="round" stroke-linejoin="round"/>
</svg>`

const platformCopy: Record<IconPlatform, { label: string; detail: string }> = {
  android: {
    label: 'Android',
    detail: 'mipmap 密度、adaptive icon、Play 图标',
  },
  ios: {
    label: 'iOS',
    detail: 'AppIcon.appiconset 与 Contents.json',
  },
  flutter: {
    label: 'Flutter',
    detail: 'android 与 ios/Runner 目录结构',
  },
  electron: {
    label: 'Electron',
    detail: 'PNG、ICO、ICNS 常用资源',
  },
  tauri: {
    label: 'Tauri',
    detail: 'src-tauri/icons 图标资源',
  },
}

const defaultPlatforms: IconPlatform[] = ['android', 'ios', 'flutter', 'electron', 'tauri']
const exportModes = [
  { id: 'icon-set', label: '生成套图' },
  { id: 'single', label: '单图导出' },
] as const
const singleFormatOptions: SvgExportFormat[] = ['png', 'jpg', 'jpeg']
const singleSizePresets = [64, 128, 256, 512, 1024]

export function SvgExportTool() {
  const runtimeReady = isTauriRuntime()
  const [exportMode, setExportMode] = useState<(typeof exportModes)[number]['id']>('icon-set')
  const [svgCode, setSvgCode] = useState(defaultSvgMarkup)
  const [appName, setAppName] = useState('app')
  const [background, setBackground] = useState('#00000000')
  const [paddingPercent, setPaddingPercent] = useState(5)
  const [outputDir, setOutputDir] = useState('')
  const [singleWidth, setSingleWidth] = useState(1024)
  const [singleHeight, setSingleHeight] = useState(1024)
  const [singleFormat, setSingleFormat] = useState<SvgExportFormat>('png')
  const [singleQuality, setSingleQuality] = useState(92)
  const [outputFilePath, setOutputFilePath] = useState('')
  const [platforms, setPlatforms] = useState<IconPlatform[]>(defaultPlatforms)
  const [isBusy, setIsBusy] = useState(false)
  const [iconSetResult, setIconSetResult] = useState<SvgIconSetResult | null>(null)
  const [singleResult, setSingleResult] = useState<SvgExportResult | null>(null)
  const [error, setError] = useState<string | null>(null)
  const previewUrl = useMemo(() => `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svgCode)}`, [svgCode])

  async function handleSelectOutputDir() {
    setError(null)
    if (!runtimeReady) {
      setError('请使用桌面应用运行后再选择输出目录。')
      return ''
    }

    const selected = await open({
      multiple: false,
      directory: true,
      title: '选择图标包输出目录',
    })

    if (!selected || Array.isArray(selected)) {
      return ''
    }

    setOutputDir(selected)
    return selected
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

  function togglePlatform(platform: IconPlatform) {
    setPlatforms((current) => {
      if (current.includes(platform)) {
        return current.filter((item) => item !== platform)
      }
      return [...current, platform]
    })
  }

  async function handleSelectOutputFile() {
    setError(null)
    if (!runtimeReady) {
      setError('请使用桌面应用运行后再选择输出文件。')
      return ''
    }

    const suggestedName = `${appName || 'icon'}-${singleWidth}x${singleHeight}.${singleFormat}`
    const selected = await save({
      title: '选择单图输出位置',
      defaultPath: outputFilePath || suggestedName,
      filters: [
        {
          name: singleFormat.toUpperCase(),
          extensions: [singleFormat === 'jpeg' ? 'jpeg' : singleFormat],
        },
      ],
    })

    if (!selected) {
      return ''
    }

    setOutputFilePath(selected)
    return selected
  }

  async function handleGenerateIconSet() {
    if (!svgCode.trim()) {
      setError('请先粘贴 SVG 代码。')
      return
    }

    if (platforms.length === 0) {
      setError('请至少选择一个目标平台。')
      return
    }

    let nextOutputDir = outputDir
    if (!nextOutputDir) {
      nextOutputDir = await handleSelectOutputDir()
    }

    if (!nextOutputDir) {
      return
    }

    const request: SvgIconSetRequest = {
      svgCode,
      outputDir: nextOutputDir,
      appName,
      platforms,
      background,
      paddingPercent,
    }

    setIsBusy(true)
    setError(null)
    setSingleResult(null)
    setIconSetResult(null)

    try {
      setIconSetResult(await exportSvgIconSet(request))
    } catch (generateError) {
      setError(getErrorMessage(generateError))
    } finally {
      setIsBusy(false)
    }
  }

  async function handleGenerateSingle() {
    if (!svgCode.trim()) {
      setError('请先粘贴 SVG 代码。')
      return
    }

    if (singleWidth <= 0 || singleHeight <= 0) {
      setError('导出尺寸必须大于 0。')
      return
    }

    let nextOutputFilePath = outputFilePath
    if (!nextOutputFilePath) {
      nextOutputFilePath = await handleSelectOutputFile()
    }

    if (!nextOutputFilePath) {
      return
    }

    const request: SvgExportRequest = {
      svgCode,
      width: singleWidth,
      height: singleHeight,
      format: singleFormat,
      quality: singleQuality,
      keepAspectRatio: true,
      background,
      paddingPercent,
      outputPath: nextOutputFilePath,
    }

    setIsBusy(true)
    setError(null)
    setIconSetResult(null)
    setSingleResult(null)

    try {
      setSingleResult(await exportSvgImage(request))
    } catch (generateError) {
      setError(getErrorMessage(generateError))
    } finally {
      setIsBusy(false)
    }
  }

  return (
    <div className="tool-stage icon-workbench">
      <div className="icon-lab-grid">
        <section className="module-card svg-source-panel">
          <div className="section-top">
            <div>
              <h4>SVG 源码</h4>
              <p>粘贴 Logo SVG，或导入本地 SVG 文件。</p>
            </div>
            <label className="secondary file-trigger">
              导入 SVG
              <input accept=".svg,image/svg+xml" onChange={handleImportFile} type="file" />
            </label>
          </div>

          <textarea
            className="code-editor icon-code-editor"
            onChange={(event) => setSvgCode(event.target.value)}
            spellCheck={false}
            value={svgCode}
          />
        </section>

        <section className="module-card icon-preview-panel">
          <div className="section-top">
            <div>
              <h4>图标预览</h4>
              <p>透明棋盘预览，便于检查边缘与留白。</p>
            </div>
          </div>
          <div className="app-icon-preview">
            <img alt="图标预览" src={previewUrl} />
          </div>
          <div className="preview-scale-row">
            <span>1024</span>
            <span>512</span>
            <span>256</span>
            <span>128</span>
            <span>64</span>
          </div>
        </section>
      </div>

      <section className="module-card icon-settings-panel">
        <div className="section-top icon-settings-head">
          <h4>生成设置</h4>
          <div className="segmented export-mode-switch">
            {exportModes.map((mode) => (
              <button
                className={exportMode === mode.id ? 'active' : ''}
                key={mode.id}
                onClick={() => {
                  setExportMode(mode.id)
                  setError(null)
                }}
                type="button"
              >
                {mode.label}
              </button>
            ))}
          </div>
        </div>

        {exportMode === 'icon-set' ? (
          <>
            <div className="icon-toolbar">
              <label className="compact-field">
                <span>应用名</span>
                <input onChange={(event) => setAppName(event.target.value)} value={appName} />
              </label>
              <label className="compact-field">
                <span>背景色</span>
                <input className="color-text-input" onChange={(event) => setBackground(event.target.value)} value={background} />
              </label>
              <label className="compact-field">
                <span>图标内边距</span>
                <input
                  max={40}
                  min={0}
                  onChange={(event) => setPaddingPercent(Number(event.target.value) || 0)}
                  type="number"
                  value={paddingPercent}
                />
              </label>
            </div>

            <div className="platform-grid platform-grid-compact">
              {defaultPlatforms.map((platform) => (
                <button
                  className={`platform-option ${platforms.includes(platform) ? 'is-selected' : ''}`}
                  key={platform}
                  onClick={() => togglePlatform(platform)}
                  type="button"
                >
                  <span>{platformCopy[platform].label}</span>
                  <small>{platformCopy[platform].detail}</small>
                </button>
              ))}
            </div>

            <div className="toolbar-actions toolbar-actions-bottom">
              <div className="output-field output-field-inline">
                <span className="output-path-prefix">目录</span>
                <input readOnly title={outputDir || '尚未选择'} value={outputDir || '尚未选择'} />
              </div>
              <button className="secondary" onClick={() => void handleSelectOutputDir()} type="button">
                选择目录
              </button>
              <button className="primary" disabled={isBusy} onClick={() => void handleGenerateIconSet()} type="button">
                {isBusy ? '生成中...' : '生成图标包'}
              </button>
            </div>
          </>
        ) : (
          <>
            <div className="single-export-grid">
              <label className="compact-field single-size-field">
                <span>宽度</span>
                <input min={1} onChange={(event) => setSingleWidth(Number(event.target.value) || 0)} type="number" value={singleWidth} />
              </label>
              <label className="compact-field single-size-field">
                <span>高度</span>
                <input min={1} onChange={(event) => setSingleHeight(Number(event.target.value) || 0)} type="number" value={singleHeight} />
              </label>
              <label className="compact-field single-format-field">
                <span>格式</span>
                <select onChange={(event) => setSingleFormat(event.target.value as SvgExportFormat)} value={singleFormat}>
                  {singleFormatOptions.map((format) => (
                    <option key={format} value={format}>
                      {format.toUpperCase()}
                    </option>
                  ))}
                </select>
              </label>
              <label className="compact-field single-background-field">
                <span>背景色</span>
                <input className="color-text-input" onChange={(event) => setBackground(event.target.value)} value={background} />
              </label>
              <label className="compact-field single-padding-field">
                <span>图标内边距</span>
                <input
                  max={40}
                  min={0}
                  onChange={(event) => setPaddingPercent(Number(event.target.value) || 0)}
                  type="number"
                  value={paddingPercent}
                />
              </label>
              <label className="compact-field single-quality-field">
                <span>JPG 质量</span>
                <input
                  disabled={singleFormat === 'png'}
                  max={100}
                  min={1}
                  onChange={(event) => setSingleQuality(Number(event.target.value) || 1)}
                  type="number"
                  value={singleQuality}
                />
              </label>
            </div>

            <div className="single-presets-row">
              <span className="single-presets-label">常用尺寸</span>
              <div className="single-presets">
                {singleSizePresets.map((size) => (
                  <button
                    className={singleWidth === size && singleHeight === size ? 'is-active' : ''}
                    key={size}
                    onClick={() => {
                      setSingleWidth(size)
                      setSingleHeight(size)
                    }}
                    type="button"
                  >
                    {size} x {size}
                  </button>
                ))}
              </div>
            </div>

            <div className="toolbar-actions toolbar-actions-bottom">
              <div className="output-field output-field-inline">
                <span className="output-path-prefix">文件</span>
                <input readOnly title={outputFilePath || '尚未选择'} value={outputFilePath || '尚未选择'} />
              </div>
              <button className="secondary" onClick={() => void handleSelectOutputFile()} type="button">
                选择文件
              </button>
              <button className="primary" disabled={isBusy} onClick={() => void handleGenerateSingle()} type="button">
                {isBusy ? '导出中...' : '导出单图'}
              </button>
            </div>
          </>
        )}
      </section>

      {error ? <p className="error-banner">{error}</p> : null}

      {iconSetResult ? (
        <section className="module-card icon-result-panel">
          <div className="section-top">
            <div>
              <h4>生成结果</h4>
              <p>{iconSetResult.outputDir}</p>
            </div>
            <div className="action-row">
              <button className="secondary" onClick={() => void openPath(iconSetResult.outputDir)} type="button">
                打开目录
              </button>
            </div>
          </div>

          <div className="dashboard-grid dashboard-grid-three">
            <article className="metric-box">
              <span>文件数量</span>
              <strong>{iconSetResult.generatedCount}</strong>
              <small>包含 PNG、XML、JSON、ICO、ICNS</small>
            </article>
            <article className="metric-box">
              <span>总大小</span>
              <strong>{formatBytes(iconSetResult.totalBytes)}</strong>
              <small>生成文件合计</small>
            </article>
            <article className="metric-box">
              <span>目标平台</span>
              <strong>{iconSetResult.platforms.length}</strong>
              <small>{iconSetResult.platforms.map((platform) => platformCopy[platform].label).join(' / ')}</small>
            </article>
          </div>
        </section>
      ) : null}

      {singleResult ? (
        <section className="module-card icon-result-panel">
          <div className="section-top">
            <div>
              <h4>导出结果</h4>
              <p>{singleResult.outputPath}</p>
            </div>
            <div className="action-row">
              <button className="secondary" onClick={() => void openPath(singleResult.outputPath)} type="button">
                打开文件
              </button>
              <button className="secondary" onClick={() => void openPath(singleResult.outputDir)} type="button">
                打开目录
              </button>
            </div>
          </div>

          <div className="dashboard-grid dashboard-grid-three">
            <article className="metric-box">
              <span>尺寸</span>
              <strong>
                {singleResult.width} x {singleResult.height}
              </strong>
              <small>按指定尺寸导出</small>
            </article>
            <article className="metric-box">
              <span>格式</span>
              <strong>{singleResult.format.toUpperCase()}</strong>
              <small>单图输出结果</small>
            </article>
            <article className="metric-box">
              <span>文件大小</span>
              <strong>{formatBytes(singleResult.outputBytes)}</strong>
              <small>单图输出结果</small>
            </article>
          </div>
        </section>
      ) : null}
    </div>
  )
}
