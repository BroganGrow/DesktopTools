# SuperTools

轻量级桌面效率工具箱，基于 `Tauri 2 + React + TypeScript + Rust` 构建。目标是把常用的图片、SVG、PDF 本地处理能力收拢到一个体积可控、启动快、跨平台的桌面应用中。

[English](./README.en.md)

![SuperTools 首页截图](./public-res/main.png)

## 功能

### 图片处理

- PNG 高清压缩：支持批量选择文件、选择文件夹和拖拽导入。
- 多格式导出：支持 `WebP`、`AVIF`、`PNG`、`JPG/JPEG`。
- 压缩策略：支持均衡、高质量、更小体积预设，并可跳过比原图更大的输出。

### SVG 图标生成

- SVG 代码导出单张图片，支持指定尺寸、格式和内边距。
- 生成 Android、iOS、Flutter、Electron、Tauri 常用图标套图。
- 支持透明背景预览和输出。

### PDF 工具

- 图片转 PDF。
- PDF 转图片。
- PDF 合并。
- PDF 拆分。
- PDF 加文本水印。

> 说明：当前 PDF 水印采用栅格化方案，会将页面渲染为图片后叠加水印再生成 PDF。适合普通内部使用；如果需要保留文本可选中、矢量清晰度和更小体积，后续应升级为 PDF overlay 方案。

## 技术栈

- 桌面框架：Tauri 2
- 前端：React、TypeScript、Vite
- 后端：Rust
- 图像编码：`cwebp`、`avifenc`
- PDF 处理：MuPDF `mutool`
- SVG 渲染：`resvg`

## 项目结构

```text
.
├─ src/                 # React 前端
├─ src-tauri/           # Tauri / Rust 后端
├─ src-tauri/binaries/  # 本地 sidecar 二进制目录，不提交具体二进制
├─ public/              # 应用静态资源
├─ public-res/          # README 等文档展示资源
└─ scripts/             # 工程脚本
```

## 本地开发

### 环境要求

- Node.js
- Rust / Cargo
- Windows 需要安装 Visual Studio C++ Build Tools
- Tauri 2 所需平台依赖

### 安装依赖

```bash
npm install
```

### 准备 sidecar 二进制

本项目通过 Tauri sidecar 调用外部命令行工具。由于二进制文件体积较大且区分平台，默认不提交到 Git。

需要放入 `src-tauri/binaries/`：

```text
cwebp-x86_64-pc-windows-msvc.exe
cwebp-x86_64-apple-darwin
cwebp-aarch64-apple-darwin

avifenc-x86_64-pc-windows-msvc.exe
avifenc-x86_64-apple-darwin
avifenc-aarch64-apple-darwin

mutool-x86_64-pc-windows-msvc.exe
mutool-x86_64-apple-darwin
mutool-aarch64-apple-darwin
```

更多说明见 [src-tauri/binaries/README.txt](./src-tauri/binaries/README.txt)。

### 启动开发版

```bash
npm run tauri:dev
```

前端开发服务默认运行在：

```text
http://127.0.0.1:1420
```

### 检查与构建

```bash
npm run check
npm run build
cd src-tauri
cargo check
```

打包桌面应用：

```bash
npm run tauri:build
```

## 当前状态

项目仍处于早期开发阶段，已完成主要工具模块的本地可用版本。签名、自动更新、公开分发、多语言、完整跨平台 sidecar 自动下载仍未纳入当前版本。

## 许可

暂未指定开源许可证。发布到 GitHub 前请根据实际用途补充 `LICENSE`。
