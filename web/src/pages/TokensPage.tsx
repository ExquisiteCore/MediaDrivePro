import { useState, useEffect } from 'react'
import { listTokens, createToken, deleteToken } from '../api/tokens'
import type { ApiTokenInfo, ApiTokenCreated } from '../api/tokens'
import { formatDateTime } from '../lib/format'
import { copyToClipboard } from '../lib/clipboard'
import { Key, Trash2, Plus, Copy, Check, Eye, EyeOff } from 'lucide-react'
import ConfirmDialog from '../components/ConfirmDialog'

export default function TokensPage() {
  const [tokens, setTokens] = useState<ApiTokenInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [deleteTarget, setDeleteTarget] = useState<ApiTokenInfo | null>(null)
  const [deleteLoading, setDeleteLoading] = useState(false)

  // Create form
  const [showCreate, setShowCreate] = useState(false)
  const [createName, setCreateName] = useState('')
  const [createLoading, setCreateLoading] = useState(false)
  const [createError, setCreateError] = useState('')

  // Created token display
  const [createdToken, setCreatedToken] = useState<ApiTokenCreated | null>(null)
  const [copied, setCopied] = useState(false)
  const [showToken, setShowToken] = useState(false)

  const load = async () => {
    try {
      const data = await listTokens()
      setTokens(data)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => { load() }, [])

  const handleCreate = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!createName.trim()) return
    setCreateError('')
    setCreateLoading(true)
    try {
      const result = await createToken({ name: createName.trim() })
      setCreatedToken(result)
      setShowCreate(false)
      setCreateName('')
      load()
    } catch (err: unknown) {
      setCreateError(err instanceof Error ? err.message : '创建失败')
    } finally {
      setCreateLoading(false)
    }
  }

  const handleDelete = async () => {
    if (!deleteTarget) return
    setDeleteLoading(true)
    try {
      await deleteToken(deleteTarget.id)
      setTokens(tokens.filter(t => t.id !== deleteTarget.id))
      setDeleteTarget(null)
    } finally {
      setDeleteLoading(false)
    }
  }

  const handleCopy = (text: string) => {
    copyToClipboard(text)
    setCopied(true)
    setTimeout(() => setCopied(false), 2000)
  }

  if (loading) {
    return <div className="p-8 text-center text-gray-400">加载中...</div>
  }

  return (
    <div className="p-6">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-gray-900">API Token</h2>
        <button
          onClick={() => setShowCreate(true)}
          className="flex items-center gap-1.5 px-3 py-1.5 text-sm text-white bg-blue-600 rounded-lg hover:bg-blue-700"
        >
          <Plus className="w-4 h-4" />
          创建 Token
        </button>
      </div>

      {/* Created token alert */}
      {createdToken && (
        <div className="mb-4 p-4 bg-green-50 border border-green-200 rounded-lg">
          <p className="text-sm font-medium text-green-800 mb-2">
            Token "{createdToken.name}" 已创建。请立即复制，关闭后将无法再次查看！
          </p>
          <div className="flex items-center gap-2 bg-white p-2 rounded border">
            <code className="flex-1 text-sm font-mono text-gray-800">
              {showToken ? createdToken.token : '••••••••••••••••••••••••••••••••'}
            </code>
            <button onClick={() => setShowToken(!showToken)} className="p-1 text-gray-500 hover:text-gray-700">
              {showToken ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
            </button>
            <button onClick={() => handleCopy(createdToken.token)} className="p-1 text-gray-500 hover:text-gray-700">
              {copied ? <Check className="w-4 h-4 text-green-600" /> : <Copy className="w-4 h-4" />}
            </button>
          </div>
          <button
            onClick={() => { setCreatedToken(null); setShowToken(false) }}
            className="mt-2 text-sm text-green-700 hover:underline"
          >
            我已复制，关闭提示
          </button>
        </div>
      )}

      {/* Create form */}
      {showCreate && (
        <div className="mb-4 p-4 bg-white border border-gray-200 rounded-lg">
          <form onSubmit={handleCreate} className="flex items-end gap-3">
            <div className="flex-1">
              <label className="block text-sm font-medium text-gray-700 mb-1">Token 名称</label>
              {createError && <p className="text-xs text-red-600 mb-1">{createError}</p>}
              <input
                type="text"
                value={createName}
                onChange={(e) => setCreateName(e.target.value)}
                placeholder="例如：WebDAV 访问"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 text-sm"
                autoFocus
              />
            </div>
            <button type="submit" disabled={createLoading || !createName.trim()} className="px-4 py-2 text-sm text-white bg-blue-600 rounded-lg hover:bg-blue-700 disabled:opacity-50">
              {createLoading ? '创建中...' : '创建'}
            </button>
            <button type="button" onClick={() => setShowCreate(false)} className="px-4 py-2 text-sm text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200">
              取消
            </button>
          </form>
        </div>
      )}

      {/* Token list */}
      {tokens.length === 0 ? (
        <div className="text-center py-16 text-gray-400">
          <Key className="w-12 h-12 mx-auto mb-3 opacity-50" />
          <p>暂无 API Token</p>
          <p className="text-sm mt-1">创建 Token 后可用于 WebDAV 等第三方访问</p>
        </div>
      ) : (
        <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
          <table className="w-full">
            <thead className="bg-gray-50 border-b border-gray-200">
              <tr>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">名称</th>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">权限</th>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">最后使用</th>
                <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">创建时间</th>
                <th className="px-4 py-2 w-20"></th>
              </tr>
            </thead>
            <tbody className="divide-y divide-gray-100">
              {tokens.map(token => (
                <tr key={token.id} className="hover:bg-gray-50">
                  <td className="px-4 py-2.5 text-sm text-gray-900 font-medium">{token.name}</td>
                  <td className="px-4 py-2.5 text-sm text-gray-500">{token.permissions}</td>
                  <td className="px-4 py-2.5 text-sm text-gray-500">
                    {token.last_used_at ? formatDateTime(token.last_used_at) : '从未'}
                  </td>
                  <td className="px-4 py-2.5 text-sm text-gray-500">{formatDateTime(token.created_at)}</td>
                  <td className="px-4 py-2.5">
                    <button
                      onClick={() => setDeleteTarget(token)}
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
        title="删除 Token"
        message={`确定要删除 Token "${deleteTarget?.name}" 吗？使用此 Token 的应用将无法继续访问。`}
        confirmText="删除"
        destructive
        loading={deleteLoading}
        onConfirm={handleDelete}
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  )
}
