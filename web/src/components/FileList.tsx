import { ArrowUp, ArrowDown } from 'lucide-react'
import FolderRow from './FolderRow'
import FileRow from './FileRow'
import type { FolderInfo } from '../api/folders'
import type { FileInfo } from '../api/files'

interface FileListProps {
  folders: FolderInfo[]
  files: FileInfo[]
  sort: string
  order: string
  onSortChange: (sort: string) => void
  onOpenFolder: (id: string) => void
  onFolderContextMenu: (folder: FolderInfo, e: React.MouseEvent) => void
  onFileContextMenu: (file: FileInfo, e: React.MouseEvent) => void
  onFilePreview: (file: FileInfo) => void
}

function SortIcon({ col, sort, order }: { col: string; sort: string; order: string }) {
  if (sort !== col) return null
  return order === 'asc'
    ? <ArrowUp className="w-3 h-3" />
    : <ArrowDown className="w-3 h-3" />
}

export default function FileList({
  folders, files, sort, order,
  onSortChange, onOpenFolder, onFolderContextMenu, onFileContextMenu, onFilePreview,
}: FileListProps) {
  const headerClass = "px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:text-gray-700 select-none"

  if (folders.length === 0 && files.length === 0) {
    return (
      <div className="text-center py-16 text-gray-400">
        <p className="text-lg mb-1">空文件夹</p>
        <p className="text-sm">上传文件或创建文件夹</p>
      </div>
    )
  }

  return (
    <table className="w-full">
      <thead className="bg-gray-50 border-b border-gray-200">
        <tr>
          <th className={headerClass} onClick={() => onSortChange('name')}>
            <span className="flex items-center gap-1">名称 <SortIcon col="name" sort={sort} order={order} /></span>
          </th>
          <th className={`${headerClass} w-28`} onClick={() => onSortChange('size')}>
            <span className="flex items-center gap-1">大小 <SortIcon col="size" sort={sort} order={order} /></span>
          </th>
          <th className={`${headerClass} w-36`} onClick={() => onSortChange('created_at')}>
            <span className="flex items-center gap-1">修改时间 <SortIcon col="created_at" sort={sort} order={order} /></span>
          </th>
          <th className={`${headerClass} w-12`}></th>
        </tr>
      </thead>
      <tbody className="divide-y divide-gray-100">
        {folders.map(folder => (
          <FolderRow
            key={folder.id}
            folder={folder}
            onOpen={() => onOpenFolder(folder.id)}
            onContextMenu={(e) => onFolderContextMenu(folder, e)}
          />
        ))}
        {files.map(file => (
          <FileRow
            key={file.id}
            file={file}
            onContextMenu={(e) => onFileContextMenu(file, e)}
            onPreview={() => onFilePreview(file)}
          />
        ))}
      </tbody>
    </table>
  )
}
