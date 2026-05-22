# SuperTools

基于 `Tauri 2 + React + TypeScript + Rust` 的轻量效率工具箱。

当前已实现的 v1 工具：

- `PNG -> WebP` 批量高清压缩
- 支持多选文件或单个文件夹输入
- 默认输出到同级新目录，不覆盖原图
- Rust 侧统一做扫描、任务调度、错误处理和进度事件

## 开发命令

```bash
npm install
npm run check
npm run lint
npm run build
npm run tauri:dev
```

## Sidecar 准备

压缩编码走 `cwebp` sidecar。由于仓库没有直接提交平台二进制，请先把对应文件放到 [src-tauri/binaries](/D:/Develop/SuperTools/src-tauri/binaries)：

- `cwebp-x86_64-pc-windows-msvc.exe`
- `cwebp-x86_64-apple-darwin`
- `cwebp-aarch64-apple-darwin`

也可以先运行：

```bash
npm run fetch:sidecars
```

这个脚本会生成一份说明文件，指向 WebP 官方发布索引：

- [WebP releases index](https://storage.googleapis.com/downloads.webmproject.org/releases/webp/index.html)

## 当前实现边界

- v1 只实际输出 `WebP`
- 文件夹模式默认只扫描当前目录，不递归子目录
- APNG 会在扫描阶段被标记为不支持并跳过
- 取消任务时，不再启动新的压缩项，已完成结果会保留

## 已验证

- `npm run check`
- `npm run lint`
- `npm run build`

## 当前阻塞

本机 Rust 工具链曾出现异常安装状态，前端构建已经验证通过，但 `cargo check` 仍依赖本机继续修正 Rust 环境后再跑一次。
