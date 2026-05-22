import { mkdir, rm, writeFile } from 'node:fs/promises'
import path from 'node:path'

const binariesDir = path.resolve('src-tauri', 'binaries')

await mkdir(binariesDir, { recursive: true })

const notice = [
  'SuperTools sidecar 准备说明',
  '',
  '当前仓库已经按 Tauri sidecar 方式接入 cwebp，但未直接提交平台二进制文件。',
  '请从 WebP 官方发布页下载对应目标的 cwebp 可执行文件，并按以下文件名放入本目录：',
  '',
  '- cwebp-x86_64-pc-windows-msvc.exe',
  '- cwebp-x86_64-apple-darwin',
  '- cwebp-aarch64-apple-darwin',
  '',
  '官方索引：',
  'https://storage.googleapis.com/downloads.webmproject.org/releases/webp/index.html',
  '',
  '建议选择同一版本的 libwebp 发布包，从其 bin 目录中取出 cwebp。',
  '如果后续需要自动化下载，可以在这个脚本里继续补充不同平台归档的下载与解压逻辑。',
  '',
].join('\n')

await rm(path.join(binariesDir, 'README.txt'), { force: true })
await writeFile(path.join(binariesDir, 'README.txt'), notice, 'utf8')

console.log(`已生成 sidecar 说明：${path.join(binariesDir, 'README.txt')}`)
