export type PreviewType = 'image' | 'video' | 'audio' | 'pdf' | 'text' | 'none'

const imageTypes = new Set([
  'image/jpeg', 'image/png', 'image/gif', 'image/webp', 'image/svg+xml', 'image/bmp',
])

const videoTypes = new Set([
  'video/mp4', 'video/webm', 'video/ogg',
])

/** Video formats that need transcoding before browser playback */
const transcodableTypes = new Set([
  'video/x-matroska', 'video/x-msvideo', 'video/quicktime',
  'video/x-flv', 'video/mpeg',
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

/** Check if a content type is a video format that needs transcoding */
export function isTranscodableVideo(contentType: string): boolean {
  return transcodableTypes.has(contentType)
}

/** Check if a content type is any video format (native or transcodable) */
export function isVideoFile(contentType: string): boolean {
  return videoTypes.has(contentType) || transcodableTypes.has(contentType)
}

export function getFileIcon(contentType: string): string {
  const type = getPreviewType(contentType)
  if (type !== 'none') return type
  if (transcodableTypes.has(contentType)) return 'video'
  return 'file'
}
