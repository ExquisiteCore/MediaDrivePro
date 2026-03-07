import { Folder } from 'lucide-react'
import { formatDate } from '../lib/format'
import type { FolderInfo } from '../api/folders'

interface FolderRowProps {
  folder: FolderInfo
  onOpen: () => void
  onContextMenu: (e: React.MouseEvent) => void
}

export default function FolderRow({ folder, onOpen, onContextMenu }: FolderRowProps) {
  return (
    <tr
      className="hover:bg-gray-50 cursor-pointer transition-colors"
      onDoubleClick={onOpen}
      onContextMenu={(e) => { e.preventDefault(); onContextMenu(e) }}
    >
      <td className="px-4 py-2.5">
        <div className="flex items-center gap-3" onClick={onOpen}>
          <Folder className="w-5 h-5 text-yellow-500 shrink-0" />
          <span className="text-sm text-gray-900 truncate">{folder.name}</span>
        </div>
      </td>
      <td className="px-4 py-2.5 text-sm text-gray-500">—</td>
      <td className="px-4 py-2.5 text-sm text-gray-500">{formatDate(folder.updated_at)}</td>
      <td className="px-4 py-2.5"></td>
    </tr>
  )
}
