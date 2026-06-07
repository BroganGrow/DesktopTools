import { open } from '@tauri-apps/plugin-dialog'
import { useEffect, useMemo, useState } from 'react'
import {
  imagesToPdf,
  isTauriRuntime,
  mergePdfs,
  openPath,
  pdfToImages,
  splitPdf,
  watermarkPdf,
} from '../lib/tauri'
import type { PdfImageFormat, PdfOperationResult, SplitPdfRequest, WatermarkPdfRequest } from '../lib/types'
import type { ToolId } from '../tool-registry'
import { getErrorMessage } from '../lib/format'

type PdfTab = 'images-to-pdf' | 'pdf-to-images' | 'merge' | 'split' | 'watermark'

const toolToTab: Partial<Record<ToolId, PdfTab>> = {
  'pdf-images-to-pdf': 'images-to-pdf',
  'pdf-to-images': 'pdf-to-images',
  'pdf-merge': 'merge',
  'pdf-split': 'split',
  'pdf-watermark': 'watermark',
}

const tabs: Array<{ id: PdfTab; label: string }> = [
  { id: 'images-to-pdf', label: '图片转 PDF' },
  { id: 'pdf-to-images', label: 'PDF 转图片' },
  { id: 'merge', label: 'PDF 合并' },
  { id: 'split', label: 'PDF 拆分' },
  { id: 'watermark', label: 'PDF 加水印' },
]

type PdfToolProps = {
  activeTool: ToolId
}

export function PdfTool({ activeTool }: PdfToolProps) {
  const runtimeReady = isTauriRuntime()
  const [activeTab, setActiveTab] = useState<PdfTab>(toolToTab[activeTool] ?? 'images-to-pdf')
  const [imagePaths, setImagePaths] = useState<string[]>([])
  const [pdfPath, setPdfPath] = useState('')
  const [pdfPaths, setPdfPaths] = useState<string[]>([])
  const [outputDir, setOutputDir] = useState('')
  const [outputName, setOutputName] = useState('output.pdf')
  const [imageFormat, setImageFormat] = useState<PdfImageFormat>('png')
  const [dpi, setDpi] = useState(144)
  const [splitMode, setSplitMode] = useState<SplitPdfRequest['mode']>('range')
  const [pageRanges, setPageRanges] = useState('1-N')
  const [watermarkText, setWatermarkText] = useState('CONFIDENTIAL')
  const [fontSize, setFontSize] = useState(48)
  const [opacity, setOpacity] = useState(0.18)
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [result, setResult] = useState<PdfOperationResult | null>(null)

  useEffect(() => {
    setActiveTab(toolToTab[activeTool] ?? 'images-to-pdf')
  }, [activeTool])

  useEffect(() => {
    setError(null)
    setResult(null)
    setOutputName(defaultOutputName(activeTab))
  }, [activeTab])

  const selectedSummary = useMemo(() => {
    if (activeTab === 'images-to-pdf') {
      return imagePaths.length ? `${imagePaths.length} 张图片` : '未选择图片'
    }
    if (activeTab === 'merge') {
      return pdfPaths.length ? `${pdfPaths.length} 个 PDF` : '未选择 PDF'
    }
    return pdfPath || '未选择 PDF'
  }, [activeTab, imagePaths.length, pdfPath, pdfPaths.length])

  async function selectImages() {
    const selected = await open({
      multiple: true,
      filters: [{ name: '图片', extensions: ['png', 'jpg', 'jpeg'] }],
      title: '选择图片',
    })
    if (!selected || typeof selected === 'string') {
      return
    }
    setImagePaths(selected)
    setOutputDir(parentDir(selected[0]))
    setOutputName('images.pdf')
  }

  async function selectSinglePdf() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
      title: '选择 PDF',
    })
    if (!selected || Array.isArray(selected)) {
      return
    }
    setPdfPath(selected)
    setOutputDir(parentDir(selected))
    setOutputName(defaultOutputName(activeTab))
  }

  async function selectMultiplePdfs() {
    const selected = await open({
      multiple: true,
      filters: [{ name: 'PDF', extensions: ['pdf'] }],
      title: '选择多个 PDF',
    })
    if (!selected || typeof selected === 'string') {
      return
    }
    setPdfPaths(selected)
    setOutputDir(parentDir(selected[0]))
    setOutputName('merged.pdf')
  }

  async function selectOutputDir() {
    const selected = await open({
      multiple: false,
      directory: true,
      title: '选择输出目录',
    })
    if (!selected || Array.isArray(selected)) {
      return
    }
    setOutputDir(selected)
  }

  async function runCurrentTool() {
    if (!runtimeReady) {
      setError('请使用桌面应用运行 PDF 工具。')
      return
    }
    if (!outputDir) {
      setError('请选择输出目录。')
      return
    }

    setBusy(true)
    setError(null)
    setResult(null)
    try {
      if (activeTab === 'images-to-pdf') {
        if (!imagePaths.length) {
          throw new Error('请选择图片。')
        }
        setResult(await imagesToPdf({ imagePaths, outputPath: joinPath(outputDir, outputName) }))
      } else if (activeTab === 'pdf-to-images') {
        if (!pdfPath) {
          throw new Error('请选择 PDF。')
        }
        setResult(await pdfToImages({ pdfPath, outputDir, format: imageFormat, dpi }))
      } else if (activeTab === 'merge') {
        if (pdfPaths.length < 2) {
          throw new Error('请选择至少两个 PDF。')
        }
        setResult(await mergePdfs({ pdfPaths, outputPath: joinPath(outputDir, outputName) }))
      } else if (activeTab === 'split') {
        if (!pdfPath) {
          throw new Error('请选择 PDF。')
        }
        setResult(await splitPdf({ pdfPath, outputDir, mode: splitMode, pageRanges }))
      } else {
        if (!pdfPath) {
          throw new Error('请选择 PDF。')
        }
        const request: WatermarkPdfRequest = {
          pdfPath,
          outputPath: joinPath(outputDir, outputName),
          text: watermarkText,
          fontSize,
          opacity,
        }
        setResult(await watermarkPdf(request))
      }
    } catch (toolError) {
      setError(getErrorMessage(toolError))
    } finally {
      setBusy(false)
    }
  }

  return (
    <div className="tool-stage pdf-workbench">
      <section className="module-card pdf-panel">
        <div className="pdf-tabs" role="tablist" aria-label="PDF 工具">
          {tabs.map((tab) => (
            <button
              className={activeTab === tab.id ? 'is-active' : ''}
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              type="button"
            >
              {tab.label}
            </button>
          ))}
        </div>

        <div className="section-top pdf-section-top">
          <div>
            <h4>{tabs.find((tab) => tab.id === activeTab)?.label}</h4>
            <p>{selectedSummary}</p>
          </div>
          <button className="secondary" onClick={selectOutputDir} type="button">
            选择输出目录
          </button>
        </div>

        <div className="pdf-form-grid">
          {activeTab === 'images-to-pdf' ? (
            <>
              <button className="secondary pdf-picker" onClick={selectImages} type="button">选择图片</button>
              <TextField label="输出文件名" value={outputName} onChange={setOutputName} />
            </>
          ) : null}

          {activeTab === 'pdf-to-images' ? (
            <>
              <button className="secondary pdf-picker" onClick={selectSinglePdf} type="button">选择 PDF</button>
              <label>
                <span>格式</span>
                <select value={imageFormat} onChange={(event) => setImageFormat(event.target.value as PdfImageFormat)}>
                  <option value="png">PNG</option>
                  <option value="jpg">JPG</option>
                </select>
              </label>
              <NumberField label="DPI" value={dpi} min={36} max={600} onChange={setDpi} />
            </>
          ) : null}

          {activeTab === 'merge' ? (
            <>
              <button className="secondary pdf-picker" onClick={selectMultiplePdfs} type="button">选择多个 PDF</button>
              <TextField label="输出文件名" value={outputName} onChange={setOutputName} />
            </>
          ) : null}

          {activeTab === 'split' ? (
            <>
              <button className="secondary pdf-picker" onClick={selectSinglePdf} type="button">选择 PDF</button>
              <label>
                <span>拆分方式</span>
                <select value={splitMode} onChange={(event) => setSplitMode(event.target.value as SplitPdfRequest['mode'])}>
                  <option value="range">指定页码范围</option>
                  <option value="pages">每页一个文件</option>
                </select>
              </label>
              {splitMode === 'range' ? <TextField label="页码范围" value={pageRanges} onChange={setPageRanges} /> : null}
            </>
          ) : null}

          {activeTab === 'watermark' ? (
            <>
              <button className="secondary pdf-picker" onClick={selectSinglePdf} type="button">选择 PDF</button>
              <TextField label="水印文本" value={watermarkText} onChange={setWatermarkText} />
              <NumberField label="字号" value={fontSize} min={12} max={160} onChange={setFontSize} />
              <NumberField label="透明度" value={opacity} min={0.05} max={1} step={0.05} onChange={setOpacity} />
              <TextField label="输出文件名" value={outputName} onChange={setOutputName} />
            </>
          ) : null}

          <label className="pdf-output-field">
            <span>输出目录</span>
            <input readOnly value={outputDir || '未选择'} />
          </label>
        </div>

        <div className="pdf-actions">
          <button className="primary" disabled={busy} onClick={() => void runCurrentTool()} type="button">
            {busy ? '处理中...' : '开始处理'}
          </button>
        </div>
      </section>

      <section className="module-card pdf-panel">
        <div className="section-top">
          <div>
            <h4>结果</h4>
            <p>{result?.message ?? '等待任务执行'}</p>
          </div>
          {result ? (
            <button className="secondary" onClick={() => void openPath(result.outputDir)} type="button">
              打开输出目录
            </button>
          ) : null}
        </div>

        {error ? <p className="error-banner">{error}</p> : null}

        {result ? (
          <div className="result-list pdf-result-list">
            {result.outputPaths.slice(0, 20).map((path) => (
              <div className="result-row pdf-result-row" key={path}>
                <code>{path}</code>
                <span className="ok">完成</span>
              </div>
            ))}
          </div>
        ) : null}
      </section>
    </div>
  )
}

type TextFieldProps = {
  label: string
  value: string
  onChange: (value: string) => void
}

function TextField({ label, value, onChange }: TextFieldProps) {
  return (
    <label>
      <span>{label}</span>
      <input value={value} onChange={(event) => onChange(event.target.value)} />
    </label>
  )
}

type NumberFieldProps = {
  label: string
  value: number
  min: number
  max: number
  step?: number
  onChange: (value: number) => void
}

function NumberField({ label, value, min, max, step = 1, onChange }: NumberFieldProps) {
  return (
    <label>
      <span>{label}</span>
      <input
        type="number"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
    </label>
  )
}

function defaultOutputName(tab: PdfTab) {
  if (tab === 'merge') {
    return 'merged.pdf'
  }
  if (tab === 'watermark') {
    return 'watermarked.pdf'
  }
  return 'output.pdf'
}

function parentDir(path: string) {
  const index = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'))
  return index >= 0 ? path.slice(0, index) : ''
}

function joinPath(dir: string, name: string) {
  const separator = dir.includes('\\') ? '\\' : '/'
  return `${dir.replace(/[\\/]+$/, '')}${separator}${name || 'output.pdf'}`
}
