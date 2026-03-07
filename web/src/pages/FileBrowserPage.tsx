import { useState, useEffect, useCallback } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { FolderPlus, Search, X } from 'lucide-react'
import Breadcrumb from '../components/Breadcrumb'
import type { BreadcrumbItem } from '../components/Breadcrumb'
import FileList from '../components/FileList'
import Pagination from '../components/Pagination'
import UploadZone from '../components/UploadZone'
import ContextMenu from '../components/ContextMenu'
import { buildFileMenuItems, buildFolderMenuItems } from '../components/menuItems'
import CreateFolderModal from '../components/CreateFolderModal'
import RenameModal from '../components/RenameModal'
import MoveModal from '../components/MoveModal'
import ShareModal from '../components/ShareModal'
import PreviewModal from '../components/PreviewModal'
import ConfirmDialog from '../components/ConfirmDialog'
import { listFiles, deleteFile, updateFile, getDownloadUrl } from '../api/files'
import type { FileInfo } from '../api/files'
import { createFolder, getFolder, listFolderChildren, listRootChildren, deleteFolder, updateFolder } from '../api/folders'
import type { FolderInfo } from '../api/folders'
import { useAuthStore } from '../store/auth'

export default function FileBrowserPage() {
  const { folderId } = useParams<{ folderId: string }>()
  const navigate = useNavigate()
  const loadUser = useAuthStore((s) => s.loadUser)

  // Data
  const [folders, setFolders] = useState<FolderInfo[]>([])
  const [files, setFiles] = useState<FileInfo[]>([])
  const [breadcrumb, setBreadcrumb] = useState<BreadcrumbItem[]>([])
  const [page, setPage] = useState(1)
  const [total, setTotal] = useState(0)
  const [perPage] = useState(20)

  // Sort & search
  const [sort, setSort] = useState('created_at')
  const [order, setOrder] = useState('desc')
  const [search, setSearch] = useState('')
  const [searchInput, setSearchInput] = useState('')

  // Modals
  const [showCreateFolder, setShowCreateFolder] = useState(false)
  const [showUpload, setShowUpload] = useState(true)
  const [renameTarget, setRenameTarget] = useState<{ id: string; name: string; type: 'file' | 'folder' } | null>(null)
  const [moveTarget, setMoveTarget] = useState<{ id: string; name: string; type: 'file' | 'folder' } | null>(null)
  const [shareTarget, setShareTarget] = useState<{ fileId?: string; folderId?: string } | null>(null)
  const [previewFile, setPreviewFile] = useState<FileInfo | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<{ id: string; name: string; type: 'file' | 'folder' } | null>(null)
  const [deleteLoading, setDeleteLoading] = useState(false)

  // Context menu
  const [contextMenu, setContextMenu] = useState<{ x: number; y: number; items: { label: string; icon: React.ReactNode; onClick: () => void; destructive?: boolean }[] } | null>(null)

  // Load data
  const loadData = useCallback(async () => {
    try {
      if (search) {
        // Search mode — files only, no folder filter
        const res = await listFiles({ search, page, per_page: perPage, sort, order })
        setFiles(res.data)
        setTotal(res.meta.total)
        setFolders([])
      } else if (folderId) {
        const children = await listFolderChildren(folderId)
        setFolders(children.folders)
        // Also load files with pagination
        const res = await listFiles({ folder_id: folderId, page, per_page: perPage, sort, order })
        setFiles(res.data)
        setTotal(res.meta.total)
      } else {
        // Root: load root folders + root files
        const children = await listRootChildren()
        setFolders(children.folders)
        setFiles(children.files)
        setTotal(children.files.length)
      }
    } catch {
      // ignore errors
    }
  }, [folderId, page, perPage, sort, order, search])

  // Build breadcrumb
  const loadBreadcrumb = useCallback(async () => {
    if (!folderId) {
      setBreadcrumb([])
      return
    }
    const crumbs: BreadcrumbItem[] = []
    let currentId: string | null = folderId
    while (currentId) {
      try {
        const folder = await getFolder(currentId)
        crumbs.unshift({ id: folder.id, name: folder.name })
        currentId = folder.parent_id
      } catch {
        break
      }
    }
    setBreadcrumb(crumbs)
  }, [folderId])

  useEffect(() => { loadData() }, [loadData])
  useEffect(() => { loadBreadcrumb() }, [loadBreadcrumb])

  // Handlers
  const handleNavigate = (id: string | null) => {
    setPage(1)
    setSearch('')
    setSearchInput('')
    if (id) {
      navigate(`/files/folder/${id}`)
    } else {
      navigate('/files')
    }
  }

  const handleSortChange = (col: string) => {
    if (sort === col) {
      setOrder(order === 'asc' ? 'desc' : 'asc')
    } else {
      setSort(col)
      setOrder(col === 'name' ? 'asc' : 'desc')
    }
    setPage(1)
  }

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    setSearch(searchInput)
    setPage(1)
  }

  const clearSearch = () => {
    setSearch('')
    setSearchInput('')
    setPage(1)
  }

  const handleCreateFolder = async (name: string) => {
    await createFolder({ name, parent_id: folderId })
    loadData()
  }

  const handleDelete = async () => {
    if (!deleteTarget) return
    setDeleteLoading(true)
    try {
      if (deleteTarget.type === 'file') {
        await deleteFile(deleteTarget.id)
      } else {
        await deleteFolder(deleteTarget.id)
      }
      loadData()
      loadUser() // refresh storage usage
      setDeleteTarget(null)
    } catch {
      // ignore
    } finally {
      setDeleteLoading(false)
    }
  }

  const handleRename = async (newName: string) => {
    if (!renameTarget) return
    if (renameTarget.type === 'file') {
      await updateFile(renameTarget.id, { name: newName })
    } else {
      await updateFolder(renameTarget.id, { name: newName })
    }
    loadData()
  }

  const handleMove = async (targetFolderId: string | null) => {
    if (!moveTarget) return
    if (moveTarget.type === 'file') {
      await updateFile(moveTarget.id, { folder_id: targetFolderId })
    } else {
      await updateFolder(moveTarget.id, { parent_id: targetFolderId })
    }
    loadData()
  }

  const handleDownload = (fileId: string) => {
    const token = localStorage.getItem('token')
    const url = `${getDownloadUrl(fileId)}?token=${encodeURIComponent(token || '')}`
    window.open(url, '_blank')
  }

  const handleFileContextMenu = (file: FileInfo, e: React.MouseEvent) => {
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      items: buildFileMenuItems({
        onPreview: () => setPreviewFile(file),
        onDownload: () => handleDownload(file.id),
        onRename: () => setRenameTarget({ id: file.id, name: file.name, type: 'file' }),
        onMove: () => setMoveTarget({ id: file.id, name: file.name, type: 'file' }),
        onShare: () => setShareTarget({ fileId: file.id }),
        onDelete: () => setDeleteTarget({ id: file.id, name: file.name, type: 'file' }),
      }),
    })
  }

  const handleFolderContextMenu = (folder: FolderInfo, e: React.MouseEvent) => {
    setContextMenu({
      x: e.clientX,
      y: e.clientY,
      items: buildFolderMenuItems({
        onRename: () => setRenameTarget({ id: folder.id, name: folder.name, type: 'folder' }),
        onMove: () => setMoveTarget({ id: folder.id, name: folder.name, type: 'folder' }),
        onDelete: () => setDeleteTarget({ id: folder.id, name: folder.name, type: 'folder' }),
      }),
    })
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-gray-200 bg-white">
        <div className="flex items-center justify-between mb-3">
          <Breadcrumb items={breadcrumb} onNavigate={handleNavigate} />
          <div className="flex items-center gap-2">
            <button
              onClick={() => setShowCreateFolder(true)}
              className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
            >
              <FolderPlus className="w-4 h-4" />
              新建文件夹
            </button>
            <button
              onClick={() => setShowUpload(!showUpload)}
              className="px-3 py-1.5 text-sm text-white bg-blue-600 rounded-lg hover:bg-blue-700 transition-colors"
            >
              上传文件
            </button>
          </div>
        </div>

        {/* Search */}
        <form onSubmit={handleSearch} className="flex items-center gap-2">
          <div className="relative flex-1 max-w-md">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
            <input
              type="text"
              value={searchInput}
              onChange={(e) => setSearchInput(e.target.value)}
              placeholder="搜索文件..."
              className="w-full pl-9 pr-3 py-1.5 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
            {search && (
              <button
                type="button"
                onClick={clearSearch}
                className="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 text-gray-400 hover:text-gray-600"
              >
                <X className="w-4 h-4" />
              </button>
            )}
          </div>
        </form>
      </div>

      {/* Upload zone */}
      {showUpload && (
        <div className="p-4 bg-white border-b border-gray-200">
          <UploadZone
            folderId={folderId}
            onUploadComplete={() => { loadData(); loadUser() }}
          />
        </div>
      )}

      {/* File list */}
      <div className="flex-1 overflow-auto bg-white">
        <FileList
          folders={search ? [] : folders}
          files={files}
          sort={sort}
          order={order}
          onSortChange={handleSortChange}
          onOpenFolder={handleNavigate}
          onFolderContextMenu={handleFolderContextMenu}
          onFileContextMenu={handleFileContextMenu}
          onFilePreview={setPreviewFile}
        />
      </div>

      {/* Pagination */}
      <Pagination page={page} total={total} perPage={perPage} onPageChange={setPage} />

      {/* Context menu */}
      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          items={contextMenu.items}
          onClose={() => setContextMenu(null)}
        />
      )}

      {/* Modals */}
      <CreateFolderModal
        open={showCreateFolder}
        onClose={() => setShowCreateFolder(false)}
        onCreate={handleCreateFolder}
      />

      {renameTarget && (
        <RenameModal
          open={true}
          currentName={renameTarget.name}
          type={renameTarget.type}
          onClose={() => setRenameTarget(null)}
          onRename={handleRename}
        />
      )}

      {moveTarget && (
        <MoveModal
          open={true}
          onClose={() => setMoveTarget(null)}
          onMove={handleMove}
          excludeId={moveTarget.type === 'folder' ? moveTarget.id : undefined}
          itemName={moveTarget.name}
        />
      )}

      {shareTarget && (
        <ShareModal
          open={true}
          fileId={shareTarget.fileId}
          folderId={shareTarget.folderId}
          onClose={() => setShareTarget(null)}
        />
      )}

      <PreviewModal
        open={!!previewFile}
        file={previewFile}
        onClose={() => setPreviewFile(null)}
      />

      <ConfirmDialog
        open={!!deleteTarget}
        title={`删除${deleteTarget?.type === 'folder' ? '文件夹' : '文件'}`}
        message={`确定要删除 "${deleteTarget?.name}" 吗？此操作不可恢复。`}
        confirmText="删除"
        destructive
        loading={deleteLoading}
        onConfirm={handleDelete}
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  )
}
