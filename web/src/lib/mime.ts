export type PreviewType = 'image' | 'video' | 'audio' | 'pdf' | 'text' | 'none'

const imageTypes = new Set([
  'image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/svg+xml', 'image/bmp',
])

const videoTypes = new Set([
  'video/mp4', 'video/webm', 'video/ogg',
])

const audioTypes = new Set([
  'audio/mpeg', 'audio/ogg', 'audio/wav', 'audio/webm', 'audio/flac', 'audio/aac',
])

const textTypes = new Set([
  'text/plain', 'text/html', 'text/css', 'text/javascript', 'text/markdown',
  'text/xml', 'text/csv', 'text/yaml',
  'application/json', 'application/xml', 'application/javascript',
  'application/x-yaml', 'application/toml',
])

export function getPreviewType(contentType: string): PreviewType {
  if (imageTypes.has(contentType)) return 'image'
  if (videoTypes.has(contentType)) return 'video'
  if (audioTypes.has(contentType)) return 'audio'
  if (contentType === 'application/pdf') return 'pdf'
  if (textTypes.has(contentType) || contentType.startsWith('text/')) return 'text'
  return 'none'
}

export function getFileIcon(contentType: string): string {
  const type = getPreviewType(contentType)
  switch (type) {
    case 'image': return 'image'
    case 'video': return 'video'
    case 'audio': return 'audio'
    case 'pdf': return 'pdf'
    case 'text': return 'text'
    default: return 'file'
  }
}
