import { useEffect, useMemo, useState } from 'react'
import './App.css'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { PngCompressTool } from './components/PngCompressTool'
import { SvgExportTool } from './components/SvgExportTool'
import { isTauriRuntime } from './lib/tauri'
import { toolRegistry, type ToolId, type ToolMeta } from './tool-registry'

const categoryLabels: Record<string, string> = {
  favorite: '常用工具',
  image: '图像处理',
  vector: '开发辅助',
  upcoming: '系统增强',
}

const topMenus = ['文件', '编辑', '视图', '窗口']
type ThemeMode = 'dark' | 'light'
const THEME_STORAGE_KEY = 'supertools-theme-mode'
type ToolCategory = keyof typeof categoryLabels
const categoryIcons: Record<ToolCategory, string> = {
  favorite: '★',
  image: 'IMG',
  vector: 'DEV',
  upcoming: 'SYS',
}

function App() {
  const [activeCategory, setActiveCategory] = useState<ToolCategory>('favorite')
  const [activeTool, setActiveTool] = useState<ToolId | null>(null)
  const [theme, setTheme] = useState<ThemeMode>(() => {
    if (typeof window === 'undefined') {
      return 'dark'
    }

    const saved = window.localStorage.getItem(THEME_STORAGE_KEY)
    if (saved === 'dark' || saved === 'light') {
      return saved
    }

    return window.matchMedia('(prefers-color-scheme: light)').matches ? 'light' : 'dark'
  })
  const [themeMenuOpen, setThemeMenuOpen] = useState(false)
  const [isMaximized, setIsMaximized] = useState(false)
  const activeMeta = toolRegistry.find((tool) => tool.id === activeTool) ?? null
  const runtimeReady = isTauriRuntime()

  const groupedTools = useMemo(() => {
    return toolRegistry.reduce<Record<string, ToolMeta[]>>((accumulator, tool) => {
      const next = accumulator[tool.category] ?? []
      next.push(tool)
      accumulator[tool.category] = next
      return accumulator
    }, {})
  }, [])

  const categoryItems = useMemo(() => {
    return (Object.keys(categoryLabels) as ToolCategory[]).map((key) => ({
      key,
      icon: categoryIcons[key],
      label: categoryLabels[key],
      count:
        key === 'favorite'
          ? toolRegistry.filter((tool) => !tool.comingSoon).length
          : groupedTools[key]?.length ?? 0,
    }))
  }, [groupedTools])

  const visibleTools =
    activeCategory === 'favorite'
      ? toolRegistry.filter((tool) => !tool.comingSoon)
      : groupedTools[activeCategory] ?? []

  useEffect(() => {
    document.documentElement.dataset.theme = theme
    window.localStorage.setItem(THEME_STORAGE_KEY, theme)
  }, [theme])

  useEffect(() => {
    if (!runtimeReady) {
      return
    }

    const appWindow = getCurrentWindow()
    let unlisten: (() => void) | undefined

    void appWindow.isMaximized().then(setIsMaximized)
    void appWindow.onResized(async () => {
      setIsMaximized(await appWindow.isMaximized())
    }).then((dispose) => {
      unlisten = dispose
    })

    return () => {
      unlisten?.()
    }
  }, [runtimeReady])

  useEffect(() => {
    function handleKeydown(event: KeyboardEvent) {
      if (event.ctrlKey && event.altKey && event.key.toLowerCase() === 'g') {
        event.preventDefault()
        setTheme((current) => (current === 'dark' ? 'light' : 'dark'))
      }

      if (event.key === 'Escape') {
        setThemeMenuOpen(false)
      }
    }

    function handlePointerDown(event: MouseEvent) {
      const target = event.target as HTMLElement | null
      if (!target?.closest('.theme-menu')) {
        setThemeMenuOpen(false)
      }
    }

    window.addEventListener('keydown', handleKeydown)
    window.addEventListener('mousedown', handlePointerDown)

    return () => {
      window.removeEventListener('keydown', handleKeydown)
      window.removeEventListener('mousedown', handlePointerDown)
    }
  }, [])

  return (
    <div className="platform-shell" data-theme={theme}>
      <header className="top-menubar">
        <div className="menubar-left">
          <div className="title-cluster" data-tauri-drag-region>
            <div className="title-mark">ST</div>
            <span className="title-name">SuperTools</span>
          </div>

          <nav className="menu-strip" aria-label="应用菜单">
            {topMenus.map((menu) => (
              <button className="menu-item" key={menu} type="button">
                {menu}
              </button>
            ))}
            <div className="theme-menu">
              <button
                aria-expanded={themeMenuOpen}
                className={`menu-item theme-trigger ${themeMenuOpen ? 'is-open' : ''}`}
                onClick={() => setThemeMenuOpen((current) => !current)}
                type="button"
              >
                主题
              </button>

              {themeMenuOpen ? (
                <div className="theme-dropdown" role="menu">
                  <button
                    className={`theme-option ${theme === 'dark' ? 'is-active' : ''}`}
                    onClick={() => {
                      setTheme('dark')
                      setThemeMenuOpen(false)
                    }}
                    type="button"
                  >
                    <span>深色主题</span>
                    <small>默认工作模式</small>
                  </button>
                  <button
                    className={`theme-option ${theme === 'light' ? 'is-active' : ''}`}
                    onClick={() => {
                      setTheme('light')
                      setThemeMenuOpen(false)
                    }}
                    type="button"
                  >
                    <span>浅色主题</span>
                    <small>明亮阅读模式</small>
                  </button>
                  <div className="theme-shortcut">
                    <span>快捷键切换</span>
                    <strong>Ctrl + Alt + G</strong>
                  </div>
                </div>
              ) : null}
            </div>
            <button className="menu-item" type="button">
              设置
            </button>
            <button className="menu-item" type="button">
              帮助
            </button>
          </nav>
        </div>

        <div className="menubar-right">
          <div className="titlebar-drag-spacer" data-tauri-drag-region />
          {runtimeReady ? (
            <div className="window-controls">
              <button
                aria-label="最小化"
                className="window-control-button"
                onClick={() => void getCurrentWindow().minimize()}
                type="button"
              >
                <span className="window-control-glyph">−</span>
              </button>
              <button
                aria-label={isMaximized ? '还原窗口' : '最大化'}
                className="window-control-button"
                onClick={async () => {
                  const appWindow = getCurrentWindow()
                  await appWindow.toggleMaximize()
                  setIsMaximized(await appWindow.isMaximized())
                }}
                type="button"
              >
                <span className="window-control-glyph">{isMaximized ? '❐' : '□'}</span>
              </button>
              <button
                aria-label="关闭"
                className="window-control-button is-close"
                onClick={() => void getCurrentWindow().close()}
                type="button"
              >
                <span className="window-control-glyph">×</span>
              </button>
            </div>
          ) : null}
        </div>
      </header>

      <div className="main-layout">
        <aside className="left-rail">
          <nav className="category-nav" aria-label="工具分类">
            {categoryItems.map((category) => (
              <button
                className={`category-item ${activeCategory === category.key ? 'is-active' : ''}`}
                key={category.key}
                onClick={() => {
                  setActiveCategory(category.key)
                  setActiveTool(null)
                }}
                type="button"
              >
                <span className="category-icon" aria-hidden="true">
                  {category.icon}
                </span>
                <span className="category-label">{category.label}</span>
                <strong>{category.count}</strong>
              </button>
            ))}
          </nav>
        </aside>

        <main className="workspace-panel shell-panel">
          {activeMeta ? (
            <>
              <header className="workspace-header">
                <div>
                  <button
                    className="back-link"
                    onClick={() => setActiveTool(null)}
                    type="button"
                  >
                    返回工具目录
                  </button>
                  <p className="eyebrow">Workspace</p>
                  <h2>{activeMeta.name}</h2>
                  <p>{activeMeta.description}</p>
                </div>

                <div className="workspace-badges">
                  <span>{activeMeta.scopeLabel}</span>
                  <span>{activeMeta.engineLabel}</span>
                  <span>{activeMeta.statusLabel}</span>
                </div>
              </header>

              <div className="workspace-body">
                {activeTool === 'png-compress' ? <PngCompressTool /> : null}
                {activeTool === 'svg-export' ? <SvgExportTool /> : null}
              </div>
            </>
          ) : (
            <>
              <header className="catalog-header catalog-header-wide">
                <div>
                  <p className="eyebrow">Tools</p>
                  <h2>{categoryLabels[activeCategory]}</h2>
                  <p>先选择工具入口，再进入具体工具页面。</p>
                </div>
                <div className="header-chip">{visibleTools.length} 个工具</div>
              </header>

              <div className="catalog-scroll workspace-home">
                <section className="catalog-group">
                  <div className="tool-grid tool-grid-wide">
                    {visibleTools.map((tool) => (
                      <button
                        className={`tool-tile tool-entry ${tool.comingSoon ? 'is-quiet' : ''}`}
                        key={tool.id}
                        onClick={() => !tool.comingSoon && setActiveTool(tool.id)}
                        type="button"
                      >
                        <div className="tool-tile-top">
                          <div className="tool-glyph" aria-hidden="true">
                            {tool.glyph}
                          </div>
                        </div>
                        <h4>{tool.name}</h4>
                        <p>{tool.summary}</p>
                      </button>
                    ))}
                  </div>
                </section>
              </div>
            </>
          )}
        </main>
      </div>
    </div>
  )
}

export default App
