export type ToolId = 'png-compress' | 'svg-export'

export type ToolMeta = {
  id: ToolId
  category: 'image' | 'vector' | 'upcoming'
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
]
