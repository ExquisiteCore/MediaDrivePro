import { useState, useEffect, useCallback } from 'react'
import { Folder, ChevronRight, ChevronDown, Home } from 'lucide-react'
import { listRootFolders } from '../api/folders'
import { listFolderChildren } from '../api/folders'
import type { FolderInfo } from '../api/folders'

interface MoveModalProps {
  open: boolean
  onClose: () => void
  onMove: (targetFolderId: string | null) => Promise<void>
  excludeId?: string
  itemName: string
}

interface FolderNode {
  folder: FolderInfo
  children: FolderNode[]
  expanded: boolean
  loaded: boolean
}

export default function MoveModal({ open, onClose, onMove, excludeId, itemName }: MoveModalProps) {
  const [roots, setRoots] = useState<FolderNode[]>([])
  const [selected, setSelected] = useState<string | null>(null)
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const loadRoots = useCallback(async () => {
    try {
      const folders = await listRootFolders()
      setRoots(folders.filter(f => f.id !== excludeId).map(f => ({
        folder: f, children: [], expanded: false, loaded: false,
      })))
    } catch { /* ignore */ }
  }, [excludeId])

  useEffect(() => {
    if (open) {
      setSelected(null)
      setError('')
      loadRoots()
    }
  }, [open, loadRoots])

  if (!open) return null

  const toggleExpand = async (nodes: FolderNode[], id: string): Promise<FolderNode[]> => {
    const result: FolderNode[] = []
    for (const node of nodes) {
      if (node.folder.id === id) {
        if (!node.loaded) {
          const children = await listFolderChildren(id)
          node.children = children.folders.filter(f => f.id !== excludeId).map(f => ({
            folder: f, children: [], expanded: false, loaded: false,
          }))
          node.loaded = true
        }
        result.push({ ...node, expanded: !node.expanded })
      } else {
        result.push({ ...node, children: await toggleExpand(node.children, id) })
      }
    }
    return result
  }

  const handleToggle = async (id: string) => {
    setRoots(await toggleExpand(roots, id))
  }

  const renderNode = (node: FolderNode, depth: number): React.ReactNode => (
    <div key={node.folder.id}>
      <div
        className={`flex items-center gap-1 py-1.5 px-2 rounded cursor-pointer transition-colors ${
          selected === node.folder.id ? 'bg-blue-50 text-blue-700' : 'hover:bg-gray-100'
        }`}
        style={{ paddingLeft: `${depth * 20 + 8}px` }}
        onClick={() => setSelected(node.folder.id)}
      >
        <button
          onClick={(e) => { e.stopPropagation(); handleToggle(node.folder.id) }}
          className="p-0.5"
        >
          {node.expanded ? <ChevronDown className="w-3 h-3" /> : <ChevronRight className="w-3 h-3" />}
        </button>
        <Folder className="w-4 h-4 text-yellow-500" />
        <span className="text-sm truncate">{node.folder.name}</span>
      </div>
      {node.expanded && node.children.map(child => renderNode(child, depth + 1))}
    </div>
  )

  const handleMove = async () => {
    setError('')
    setLoading(true)
    try {
      await onMove(selected)
      onClose()
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : '移动失败')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={onClose}>
      <div className="bg-white rounded-xl shadow-lg p-6 w-full max-w-sm" onClick={(e) => e.stopPropagation()}>
        <h3 className="text-lg font-semibold text-gray-900 mb-1">移动</h3>
        <p className="text-sm text-gray-500 mb-4">将 "{itemName}" 移动到：</p>

        {error && <div className="text-sm text-red-600 bg-red-50 p-2 rounded mb-3">{error}</div>}

        <div className="border border-gray-200 rounded-lg max-h-64 overflow-auto mb-4">
          <div
            className={`flex items-center gap-2 py-1.5 px-2 rounded cursor-pointer transition-colors ${
              selected === null ? 'bg-blue-50 text-blue-700' : 'hover:bg-gray-100'
            }`}
            onClick={() => setSelected(null)}
          >
            <Home className="w-4 h-4" />
            <span className="text-sm">根目录</span>
          </div>
          {roots.map(node => renderNode(node, 1))}
        </div>

        <div className="flex justify-end gap-3">
          <button onClick={onClose} className="px-4 py-2 text-sm text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200">
            取消
          </button>
          <button onClick={handleMove} disabled={loading} className="px-4 py-2 text-sm text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50">
            {loading ? '移动中...' : '移动到此'}
          </button>
        </div>
      </div>
    </div>
  )
}
