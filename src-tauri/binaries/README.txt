SuperTools sidecar 目录

请将以下可执行文件放入此目录，并使用 Tauri externalBin 约定的目标命名：

1) WebP 编码器 cwebp
- cwebp-x86_64-pc-windows-msvc.exe
- cwebp-x86_64-apple-darwin
- cwebp-aarch64-apple-darwin

官方发布索引：
https://storage.googleapis.com/downloads.webmproject.org/releases/webp/index.html

建议从同一版本的 libwebp 发布包中提取 `bin/cwebp`。

2) AVIF 编码器 avifenc（启用 AVIF 时需要）
- avifenc-x86_64-pc-windows-msvc.exe
- avifenc-x86_64-apple-darwin
- avifenc-aarch64-apple-darwin

官方仓库：
https://github.com/AOMediaCodec/libavif

建议从 libavif 发布产物中提取 `avifenc`，尽量保证三平台版本一致。
当前仓库的 macOS 两个文件来自 `v1.4.1/macOS-artifacts.zip` 中的 `avifenc`（可用于 Intel / Apple Silicon 打包）。

注意：
- 当前配置已包含 `binaries/avifenc` 到 `src-tauri/tauri.conf.json > bundle.externalBin`。

3) PDF 工具 mutool
- mutool-x86_64-pc-windows-msvc.exe
- mutool-x86_64-apple-darwin
- mutool-aarch64-apple-darwin

官方发布：
https://mupdf.com/releases/

当前 Windows 文件来自 MuPDF 1.27.0 Windows 发布包，用于 PDF 合并、拆分、转图片等操作。
