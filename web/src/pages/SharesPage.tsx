import { useState, useEffect } from 'react'
import { listShares, deleteShare } from '../api/shares'
import type { ShareInfo } from '../api/shares'
import { formatDateTime } from '../lib/format'
import { Copy, Check, Trash2, Link } from 'lucide-react'
import ConfirmDialog from '../components/ConfirmDialog'

export default function SharesPage() {
  const [shares, setShares] = useState<ShareInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [deleteTarget, setDeleteTarget] = useState<ShareInfo | null>(null)
  const [deleteLoading, setDeleteLoading] = useState(false)
  const [copiedId, setCopiedId] = useState<string | null>(null)

  const load = async () => {
    try {
      const data = await listShares()
      setShares(data)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  const handleDelete = async () => {
    if (!deleteTarget) return
    setDeleteLoading(true)
    try {
      await deleteShare(deleteTarget.id)
      setShares(shares.filter(s => s.id !== deleteTarget.id))
      setDeleteTarget(null)
    } finally {
      setDeleteLoading(false)
    }
  }

  const handleCopy = async (token: string, id: string) => {
    await navigator.clipboard.writeText(`${window.location.origin}/s/${token}`)
    setCopiedId(id)
    setTimeout(() => setCopiedId(null), 2000)
  }

  if (loading) {
    return <div className="p-8 text-center text-gray-400">加载中...</div>
  }

  return (
    <div className="p-6">
      <h2 className="text-xl font-semibold text-gray-900 mb-4">分享管理</h2>

      {shares.length === 0 ? (
        <div className="text-center py-16 text-gray-400">
          <Link className="w-12 h-12 mx-auto mb-3 opacity-50" />
          <p>暂无分享</p>
          <p className="text-sm mt-1">在文件浏览器中右键文件即可创建分享</p>
        </div>
      ) : (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">分享链接</th>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">密码</th>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">下载</th>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">过期</th>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">创建时间</th>
                <th className="px-4 py-2 w-20"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {shares.map(share => (
                <tr key={share.id} className="hover:bg-gray-50">
                  <td className="px-4 py-2.5">
                    <div className="flex items-center gap-2">
                      <code className="text-sm text-blue-600">/s/{share.token}</code>
                      <button
                        onClick={() => handleCopy(share.token, share.id)}
                        className="p-1 text-gray-400 hover:text-gray-600"
                      >
                        {copiedId === share.id ? <Check className="w-3.5 h-3.5 text-green-600" /> : <Copy className="w-3.5 h-3.5" />}
                      </button>
                    </div>
                  </td>
                  <td className="px-4 py-2.5 text-sm text-gray-500">
                    {share.has_password ? '是' : '否'}
                  </td>
                  <td className="px-4 py-2.5 text-sm text-gray-500">
                    {share.download_count}{share.max_downloads ? ` / ${share.max_downloads}` : ''}
                  </td>
                  <td className="px-4 py-2.5 text-sm text-gray-500">
                    {share.expires_at ? formatDateTime(share.expires_at) : '永不'}
                  </td>
                  <td className="px-4 py-2.5 text-sm text-gray-500">
                    {formatDateTime(share.created_at)}
                  </td>
                  <td className="px-4 py-2.5">
                    <button
                      onClick={() => setDeleteTarget(share)}
                      className="p-1 text-gray-400 hover:text-red-500 transition-colors"
                    >
                      <Trash2 className="w-4 h-4" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      <ConfirmDialog
        open={!!deleteTarget}
        title="删除分享"
        message="确定要删除此分享链接吗？删除后他人将无法通过此链接访问。"
        confirmText="删除"
        destructive
        loading={deleteLoading}
        onConfirm={handleDelete}
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  )
}
