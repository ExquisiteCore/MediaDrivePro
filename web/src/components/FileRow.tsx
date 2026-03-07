import {
  File, Image, Video, Music, FileText, FileCode,
} from 'lucide-react'
import { formatFileSize, formatDate } from '../lib/format'
import { getPreviewType, isTranscodableVideo } from '../lib/mime'
import type { FileInfo } from '../api/files'

interface FileRowProps {
  file: FileInfo
  onContextMenu: (e: React.MouseEvent) => void
  onPreview: () => void
}

function FileIcon({ contentType }: { contentType: string }) {
  const type = getPreviewType(contentType)
  const transcodable = isTranscodableVideo(contentType)
  const cls = "w-5 h-5 shrink-0"
  if (type === 'video' || transcodable) return <Video className={`${cls} text-purple-500`} />
  switch (type) {
    case 'image': return <Image className={`${cls} text-green-500`} />
    case 'audio': return <Music className={`${cls} text-pink-500`} />
    case 'pdf': return <FileCode className={`${cls} text-red-500`} />
    case 'text': return <FileText className={`${cls} text-blue-500`} />
    default: return <File className={`${cls} text-gray-400`} />
  }
}

export default function FileRow({ file, onContextMenu, onPreview }: FileRowProps) {
  const previewable = getPreviewType(file.content_type) !== 'none' || isTranscodableVideo(file.content_type)

  return (
    <tr
      className="hover:bg-gray-50 transition-colors"
      onContextMenu={(e) => { e.preventDefault(); onContextMenu(e) }}
      onDoubleClick={previewable ? onPreview : undefined}
    >
      <td className="px-4 py-2.5">
        <div className={`flex items-center gap-3 ${previewable ? 'cursor-pointer' : ''}`} onClick={previewable ? onPreview : undefined}>
          <FileIcon contentType={file.content_type} />
          <span className="text-sm text-gray-900 truncate">{file.name}</span>
        </div>
      </td>
      <td className="px-4 py-2.5 text-sm text-gray-500">{formatFileSize(file.size)}</td>
      <td className="px-4 py-2.5 text-sm text-gray-500">{formatDate(file.updated_at)}</td>
      <td className="px-4 py-2.5"></td>
    </tr>
  )
}
