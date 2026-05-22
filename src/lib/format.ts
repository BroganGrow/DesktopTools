export function formatBytes(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`
  }

  const units = ['KB', 'MB', 'GB']
  let value = bytes / 1024
  let index = 0

  while (value >= 1024 && index < units.length - 1) {
    value /= 1024
    index += 1
  }

  return `${value.toFixed(value >= 100 ? 0 : 1)} ${units[index]}`
}

export function formatDuration(ms: number) {
  if (ms < 1000) {
    return `${ms} ms`
  }

  return `${(ms / 1000).toFixed(1)} s`
}

export function getErrorMessage(error: unknown) {
  if (error instanceof Error) {
    return error.message
  }

  if (typeof error === 'string') {
    return error
  }

  return '发生了未知错误。'
}
