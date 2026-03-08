import { useCallback, useEffect, useRef, useState } from 'react'
import { listImages, uploadImage, deleteImage } from '../api/images'
import type { ImageInfo } from '../api/images'
import { formatFileSize, formatDateTime } from '../lib/format'
import { copyToClipboard } from '../lib/clipboard'
import { Upload, Trash2, Copy, Check, X, ImageIcon, ChevronLeft, ChevronRight } from 'lucide-react'

export default function ImagesPage() {
  const [images, setImages] = useState<ImageInfo[]>([])
  const [page, setPage] = useState(1)
  const [total, setTotal] = useState(0)
  const [loading, setLoading] = useState(false)
  const [uploading, setUploading] = useState(false)
  const [selected, setSelected] = useState<ImageInfo | null>(null)
  const [copied, setCopied] = useState('')
  const [error, setError] = useState('')
  const fileInputRef = useRef<HTMLInputElement>(null)
  const perPage = 20

  const fetchImages = useCallback(async () => {
    setLoading(true)
    try {
      const res = await listImages(page, perPage)
      setImages(res.data)
      setTotal(res.meta.total)
    } catch (err) {
      setError(err instanceof Error ? err.message : '加载失败')
    } finally {
      setLoading(false)
    }
  }, [page])

  useEffect(() => {
    fetchImages()
  }, [fetchImages])

  const handleUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files
    if (!files?.length) return

    setUploading(true)
    setError('')
    try {
      for (const file of Array.from(files)) {
        await uploadImage(file)
      }
      setPage(1)
      await fetchImages()
    } catch (err) {
      setError(err instanceof Error ? err.message : '上传失败')
    } finally {
      setUploading(false)
      if (fileInputRef.current) fileInputRef.current.value = ''
    }
  }

  const handleDelete = async (id: string) => {
    if (!confirm('确定删除这张图片？')) return
    try {
      await deleteImage(id)
      setSelected(null)
      await fetchImages()
    } catch (err) {
      setError(err instanceof Error ? err.message : '删除失败')
    }
  }

  const copyText = (text: string, label: string) => {
    copyToClipboard(text)
    setCopied(label)
    setTimeout(() => setCopied(''), 2000)
  }

  const totalPages = Math.ceil(total / perPage)

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-6">
        <h2 className="text-xl font-semibold text-gray-900">图床</h2>
        <button
          onClick={() => fileInputRef.current?.click()}
          disabled={uploading}
          className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition-colors text-sm"
        >
          <Upload className="w-4 h-4" />
          {uploading ? '上传中...' : '上传图片'}
        </button>
        <input
          ref={fileInputRef}
          type="file"
          accept="image/jpeg,image/png,image/gif,image/webp"
          multiple
          onChange={handleUpload}
          className="hidden"
        />
      </div>

      {error && (
        <div className="text-sm text-red-600 bg-red-50 p-3 rounded-lg mb-4">{error}</div>
      )}

      {loading && images.length === 0 ? (
        <div className="text-center text-gray-400 py-20">加载中...</div>
      ) : images.length === 0 ? (
        <div className="text-center py-20">
          <ImageIcon className="w-12 h-12 text-gray-300 mx-auto mb-3" />
          <p className="text-gray-400">还没有上传图片</p>
          <p className="text-sm text-gray-400 mt-1">点击上方按钮上传第一张图片</p>
        </div>
      ) : (
        <>
          <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-3">
            {images.map((img) => (
              <div
                key={img.id}
                onClick={() => setSelected(img)}
                className="group relative aspect-square rounded-lg overflow-hidden bg-gray-100 cursor-pointer border border-gray-200 hover:border-blue-400 transition-colors"
              >
                <img
                  src={img.thumb_url}
                  alt={img.original_name}
                  className="w-full h-full object-cover"
                  loading="lazy"
                />
                <div className="absolute inset-0 bg-black/0 group-hover:bg-black/10 transition-colors" />
              </div>
            ))}
          </div>

          {totalPages > 1 && (
            <div className="flex items-center justify-center gap-2 mt-6">
              <button
                onClick={() => setPage((p) => Math.max(1, p - 1))}
                disabled={page <= 1}
                className="p-2 rounded-lg hover:bg-gray-100 disabled:opacity-30 transition-colors"
              >
                <ChevronLeft className="w-4 h-4" />
              </button>
              <span className="text-sm text-gray-600">
                {page} / {totalPages}
              </span>
              <button
                onClick={() => setPage((p) => Math.min(totalPages, p + 1))}
                disabled={page >= totalPages}
                className="p-2 rounded-lg hover:bg-gray-100 disabled:opacity-30 transition-colors"
              >
                <ChevronRight className="w-4 h-4" />
              </button>
            </div>
          )}
        </>
      )}

      {/* Detail modal */}
      {selected && (
        <div
          className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
          onClick={() => setSelected(null)}
        >
          <div
            className="bg-white rounded-xl max-w-2xl w-full max-h-[90vh] overflow-auto"
            onClick={(e) => e.stopPropagation()}
          >
            <div className="flex items-center justify-between p-4 border-b border-gray-200">
              <h3 className="font-medium text-gray-900 truncate">{selected.original_name}</h3>
              <button
                onClick={() => setSelected(null)}
                className="p-1 hover:bg-gray-100 rounded transition-colors"
              >
                <X className="w-5 h-5 text-gray-400" />
              </button>
            </div>

            <div className="p-4">
              <div className="bg-gray-50 rounded-lg p-2 mb-4 flex items-center justify-center" style={{ maxHeight: 400 }}>
                <img
                  src={selected.url}
                  alt={selected.original_name}
                  className="max-w-full max-h-[380px] object-contain rounded"
                />
              </div>

              <div className="space-y-2 mb-4">
                <CopyRow
                  label="URL"
                  value={selected.url}
                  copied={copied}
                  onCopy={copyText}
                />
                <CopyRow
                  label="Markdown"
                  value={selected.markdown}
                  copied={copied}
                  onCopy={copyText}
                />
                <CopyRow
                  label="HTML"
                  value={`<img src="${selected.url}" alt="${selected.original_name}" />`}
                  copied={copied}
                  onCopy={copyText}
                />
                <CopyRow
                  label="BBCode"
                  value={`[img]${selected.url}[/img]`}
                  copied={copied}
                  onCopy={copyText}
                />
              </div>

              <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-sm text-gray-500 mb-4">
                <div>尺寸: {selected.width} x {selected.height}</div>
                <div>压缩后: {formatFileSize(selected.size)}</div>
                <div>原始大小: {formatFileSize(selected.original_size)}</div>
                <div>上传时间: {formatDateTime(selected.created_at)}</div>
              </div>

              <button
                onClick={() => handleDelete(selected.id)}
                className="flex items-center gap-2 px-3 py-1.5 text-sm text-red-600 hover:bg-red-50 rounded-lg transition-colors"
              >
                <Trash2 className="w-4 h-4" />
                删除图片
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}

function CopyRow({
  label,
  value,
  copied,
  onCopy,
}: {
  label: string
  value: string
  copied: string
  onCopy: (text: string, label: string) => void
}) {
  const isCopied = copied === label
  return (
    <div className="flex items-center gap-2">
      <span className="text-xs text-gray-400 w-16 shrink-0">{label}</span>
      <div className="flex-1 min-w-0 bg-gray-50 rounded px-2 py-1 text-sm text-gray-700 font-mono truncate">
        {value}
      </div>
      <button
        onClick={() => onCopy(value, label)}
        className="p-1.5 hover:bg-gray-100 rounded transition-colors shrink-0"
        title="复制"
      >
        {isCopied ? (
          <Check className="w-4 h-4 text-green-500" />
        ) : (
          <Copy className="w-4 h-4 text-gray-400" />
        )}
      </button>
    </div>
  )
}
