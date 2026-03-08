import { useState } from 'react'
import { createShare } from '../api/shares'
import { copyToClipboard } from '../lib/clipboard'
import { Copy, Check } from 'lucide-react'

interface ShareModalProps {
  open: boolean
  fileId?: string
  folderId?: string
  onClose: () => void
}

export default function ShareModal({ open, fileId, folderId, onClose }: ShareModalProps) {
  const [password, setPassword] = useState('')
  const [maxDownloads, setMaxDownloads] = useState('')
  const [expiresIn, setExpiresIn] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  const [shareUrl, setShareUrl] = useState('')
  const [copied, setCopied] = useState(false)

  if (!open) return null

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)
    try {
      let expires_at: string | undefined
      if (expiresIn) {
        const d = new Date()
        d.setHours(d.getHours() + parseInt(expiresIn))
        expires_at = d.toISOString()
      }
      const share = await createShare({
        file_id: fileId,
        folder_id: folderId,
        password: password || undefined,
        max_downloads: maxDownloads ? parseInt(maxDownloads) : undefined,
        expires_at,
      })
      setShareUrl(`${window.location.origin}/s/${share.token}`)
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : '创建分享失败')
    } finally {
      setLoading(false)
    }
  }

  const handleCopy = () => {
    copyToClipboard(shareUrl)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  const handleClose = () => {
    setShareUrl('')
    setPassword('')
    setMaxDownloads('')
    setExpiresIn('')
    setError('')
    setCopied(false)
    onClose()
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/40" onClick={handleClose}>
      <div className="bg-white rounded-xl shadow-lg p-6 w-full max-w-sm" onClick={(e) => e.stopPropagation()}>
        <h3 className="text-lg font-semibold text-gray-900 mb-4">创建分享链接</h3>

        {shareUrl ? (
          <div>
            <p className="text-sm text-gray-600 mb-2">分享链接已创建：</p>
            <div className="flex items-center gap-2 bg-gray-50 p-3 rounded-lg">
              <input
                type="text"
                value={shareUrl}
                readOnly
                className="flex-1 text-sm bg-transparent border-none outline-none"
              />
              <button onClick={handleCopy} className="p-1 text-gray-500 hover:text-blue-600">
                {copied ? <Check className="w-4 h-4 text-green-600" /> : <Copy className="w-4 h-4" />}
              </button>
            </div>
            <button
              onClick={handleClose}
              className="w-full mt-4 px-4 py-2 text-sm text-white bg-blue-600 rounded-lg hover:bg-blue-700"
            >
              完成
            </button>
          </div>
        ) : (
          <form onSubmit={handleSubmit}>
            {error && <div className="text-sm text-red-600 bg-red-50 p-2 rounded mb-3">{error}</div>}

            <div className="space-y-3 mb-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">密码保护（可选）</label>
                <input
                  type="text"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="留空则无需密码"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">最大下载次数（可选）</label>
                <input
                  type="number"
                  value={maxDownloads}
                  onChange={(e) => setMaxDownloads(e.target.value)}
                  placeholder="不限制"
                  min="1"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">过期时间</label>
                <select
                  value={expiresIn}
                  onChange={(e) => setExpiresIn(e.target.value)}
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent text-sm"
                >
                  <option value="">永不过期</option>
                  <option value="1">1 小时</option>
                  <option value="24">1 天</option>
                  <option value="168">7 天</option>
                  <option value="720">30 天</option>
                </select>
              </div>
            </div>

            <div className="flex justify-end gap-3">
              <button type="button" onClick={handleClose} className="px-4 py-2 text-sm text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200">
                取消
              </button>
              <button type="submit" disabled={loading} className="px-4 py-2 text-sm text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50">
                {loading ? '创建中...' : '创建分享'}
              </button>
            </div>
          </form>
        )}
      </div>
    </div>
  )
}
