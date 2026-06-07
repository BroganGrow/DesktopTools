export type ToolId =
  | 'png-compress'
  | 'svg-export'
  | 'pdf-images-to-pdf'
  | 'pdf-to-images'
  | 'pdf-merge'
  | 'pdf-split'
  | 'pdf-watermark'

export type ToolMeta = {
  id: ToolId
  category: 'image' | 'vector' | 'pdf' | 'upcoming'
  glyph: string
  name: string
  summary: string
  description: string
  statusLabel: string
  statusTone: 'ready' | 'new'
  scopeLabel: string
  engineLabel: string
  comingSoon?: boolean
}

export const toolRegistry: ToolMeta[] = [
  {
    id: 'png-compress',
    category: 'image',
    glyph: 'PX',
    name: 'PNG 高清压缩',
    summary: '批量把 PNG 压缩成 WebP，保留尺寸和透明通道。',
    description: '面向素材整理和资源瘦身的批量压缩模块，默认不碰原图，适合做第一批生产工具。',
    statusLabel: '已上线',
    statusTone: 'ready',
    scopeLabel: '批量任务',
    engineLabel: 'cwebp sidecar',
  },
  {
    id: 'svg-export',
    category: 'vector',
    glyph: 'SV',
    name: 'SVG 图标生成',
    summary: '从 Logo SVG 生成 Android、iOS、Flutter、Electron、Tauri 图标包。',
    description: '面向多端工程的图标资源生成模块，适合把设计稿里的 SVG Logo 快速转换成项目可用资源。',
    statusLabel: '已上线',
    statusTone: 'new',
    scopeLabel: '多端图标包',
    engineLabel: 'resvg + png/ico/icns',
  },
  {
    id: 'pdf-images-to-pdf',
    category: 'pdf',
    glyph: 'I2P',
    name: '图片转 PDF',
    summary: '多张图片合成为一个 PDF。',
    description: '适合把截图、扫描件、设计稿快速整理成 PDF 文档。',
    statusLabel: '已上线',
    statusTone: 'new',
    scopeLabel: 'PDF',
    engineLabel: 'Rust PDF writer',
  },
  {
    id: 'pdf-to-images',
    category: 'pdf',
    glyph: 'P2I',
    name: 'PDF 转图片',
    summary: '把 PDF 页面导出为 PNG 或 JPG。',
    description: '按页渲染 PDF，适合生成预览图、页面素材或归档图片。',
    statusLabel: '已上线',
    statusTone: 'new',
    scopeLabel: 'PDF',
    engineLabel: 'mutool',
  },
  {
    id: 'pdf-merge',
    category: 'pdf',
    glyph: 'PDF',
    name: 'PDF 合并',
    summary: '多个 PDF 按顺序合并为一个文件。',
    description: '支持选择多个 PDF，并按选择顺序合成为单个输出文件。',
    statusLabel: '已上线',
    statusTone: 'new',
    scopeLabel: 'PDF',
    engineLabel: 'mutool',
  },
  {
    id: 'pdf-split',
    category: 'pdf',
    glyph: 'CUT',
    name: 'PDF 拆分',
    summary: '按页码范围或逐页拆分 PDF。',
    description: '支持指定页码范围输出，也支持把 PDF 每页拆成独立文件。',
    statusLabel: '已上线',
    statusTone: 'new',
    scopeLabel: 'PDF',
    engineLabel: 'mutool',
  },
  {
    id: 'pdf-watermark',
    category: 'pdf',
    glyph: 'WM',
    name: 'PDF 加水印',
    summary: '给 PDF 页面添加文本水印。',
    description: '本地写入文本水印，适合内部流转、标记版本或添加简单声明。',
    statusLabel: '已上线',
    statusTone: 'new',
    scopeLabel: 'PDF',
    engineLabel: 'mutool + resvg',
  },
]
