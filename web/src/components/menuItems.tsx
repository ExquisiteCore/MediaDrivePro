import { Download, Pencil, FolderInput, Share2, Trash2, Eye } from 'lucide-react'

interface MenuItem {
  label: string
  icon: React.ReactNode
  onClick: () => void
  destructive?: boolean
}

export function buildFileMenuItems(actions: {
  onPreview?: () => void
  onDownload?: () => void
  onRename?: () => void
  onMove?: () => void
  onShare?: () => void
  onDelete?: () => void
}) {
  const items: MenuItem[] = []
  if (actions.onPreview) items.push({ label: '预览', icon: <Eye className="w-4 h-4" />, onClick: actions.onPreview })
  if (actions.onDownload) items.push({ label: '下载', icon: <Download className="w-4 h-4" />, onClick: actions.onDownload })
  if (actions.onRename) items.push({ label: '重命名', icon: <Pencil className="w-4 h-4" />, onClick: actions.onRename })
  if (actions.onMove) items.push({ label: '移动到', icon: <FolderInput className="w-4 h-4" />, onClick: actions.onMove })
  if (actions.onShare) items.push({ label: '分享', icon: <Share2 className="w-4 h-4" />, onClick: actions.onShare })
  if (actions.onDelete) items.push({ label: '删除', icon: <Trash2 className="w-4 h-4" />, onClick: actions.onDelete, destructive: true })
  return items
}

export function buildFolderMenuItems(actions: {
  onRename?: () => void
  onMove?: () => void
  onDelete?: () => void
}) {
  const items: MenuItem[] = []
  if (actions.onRename) items.push({ label: '重命名', icon: <Pencil className="w-4 h-4" />, onClick: actions.onRename })
  if (actions.onMove) items.push({ label: '移动到', icon: <FolderInput className="w-4 h-4" />, onClick: actions.onMove })
  if (actions.onDelete) items.push({ label: '删除', icon: <Trash2 className="w-4 h-4" />, onClick: actions.onDelete, destructive: true })
  return items
}
