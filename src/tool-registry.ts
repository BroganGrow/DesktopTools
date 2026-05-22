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
    name: 'SVG 代码导图',
    summary: '从 SVG 代码导出指定尺寸的 PNG、JPG、JPEG。',
    description: '面向图标、海报和前端素材的出图模块，支持实时预览和目标画布预览。',
    statusLabel: '已上线',
    statusTone: 'new',
    scopeLabel: '单文件生成',
    engineLabel: 'resvg + image',
  },
]
