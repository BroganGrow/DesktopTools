$root = Split-Path -Parent $PSScriptRoot
$iconsDir = Join-Path $root 'src-tauri/icons'
$publicDir = Join-Path $root 'public'
New-Item -ItemType Directory -Force -Path $iconsDir | Out-Null
New-Item -ItemType Directory -Force -Path $publicDir | Out-Null

function New-SinglePngIco {
  param(
    [string]$PngPath,
    [string]$OutputPath
  )

  $bytes = [System.IO.File]::ReadAllBytes($PngPath)
  $stream = New-Object System.IO.MemoryStream
  $writer = New-Object System.IO.BinaryWriter $stream
  $writer.Write([UInt16]0)
  $writer.Write([UInt16]1)
  $writer.Write([UInt16]1)
  $writer.Write([byte]0)
  $writer.Write([byte]0)
  $writer.Write([byte]0)
  $writer.Write([byte]0)
  $writer.Write([UInt16]1)
  $writer.Write([UInt16]32)
  $writer.Write([UInt32]$bytes.Length)
  $writer.Write([UInt32]22)
  $writer.Write($bytes)
  [System.IO.File]::WriteAllBytes($OutputPath, $stream.ToArray())
  $writer.Dispose()
  $stream.Dispose()
}

$png1024 = Join-Path $iconsDir 'icon.png'
$png256 = Join-Path $iconsDir '128x128@2x.png'

Copy-Item $png1024 (Join-Path $publicDir 'logo-mark.png') -Force
New-SinglePngIco $png256 (Join-Path $iconsDir 'icon.ico')
