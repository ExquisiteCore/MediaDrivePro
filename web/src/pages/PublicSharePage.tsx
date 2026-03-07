import { useState, useEffect } from 'react'
import { useParams } from 'react-router-dom'
import { getPublicShare, verifySharePassword, getShareDownloadUrl } from '../api/shares'
import type { PublicShareInfo } from '../api/shares'
import { formatFileSize } from '../lib/format'
import { Download, Lock, FileIcon, HardDrive } from 'lucide-react'

export default function PublicSharePage() {
  const { token } = useParams<{ token: string }>()
  const [share, setShare] = useState<PublicShareInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [needPassword, setNeedPassword] = useState(false)
  const [password, setPassword] = useState('')
  const [passwordError, setPasswordError] = useState('')
  const [verified, setVerified] = useState(false)

  useEffect(() => {
    if (!token) return
    getPublicShare(token)
      .then(info => {
        setShare(info)
        if (info.has_password) {
          setNeedPassword(true)
        }
      })
      .catch(err => setError(err instanceof Error ? err.message : '分享不存在'))
      .finally(() => setLoading(false))
  }, [token])

  const handleVerify = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!token) return
    setPasswordError('')
    try {
      await verifySharePassword(token, password)
      setVerified(true)
      setNeedPassword(false)
    } catch (err: unknown) {
      setPasswordError(err instanceof Error ? err.message : '密码错误')
    }
  }

  const handleDownload = () => {
    if (!token) return
    window.open(getShareDownloadUrl(token), '_blank')
  }

  if (loading) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <p className="text-gray-400">加载中...</p>
      </div>
    )
  }

  if (error) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-gray-50">
        <div className="text-center">
          <HardDrive className="w-12 h-12 mx-auto text-gray-300 mb-4" />
          <p className="text-gray-600">{error}</p>
        </div>
      </div>
    )
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50">
      <div className="w-full max-w-sm">
        <div className="flex items-center justify-center gap-2 mb-8">
          <HardDrive className="w-6 h-6 text-blue-600" />
          <span className="text-lg font-bold text-gray-900">MediaDrive</span>
        </div>

        <div className="bg-white p-6 rounded-xl shadow-sm border border-gray-200">
          {needPassword && !verified ? (
            <div>
              <div className="flex items-center gap-2 mb-4">
                <Lock className="w-5 h-5 text-gray-400" />
                <h3 className="text-lg font-semibold text-gray-900">需要密码</h3>
              </div>
              <form onSubmit={handleVerify}>
                {passwordError && (
                  <div className="text-sm text-red-600 bg-red-50 p-2 rounded mb-3">{passwordError}</div>
                )}
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="输入分享密码"
                  className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 mb-4"
                  autoFocus
                />
                <button type="submit" className="w-full py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700">
                  验证
                </button>
              </form>
            </div>
          ) : (
            <div>
              <div className="flex items-center gap-3 mb-4">
                <FileIcon className="w-10 h-10 text-gray-400" />
                <div>
                  <h3 className="font-semibold text-gray-900 truncate">{share?.file_name || '未知文件'}</h3>
                  {share?.file_size !== null && share?.file_size !== undefined && (
                    <p className="text-sm text-gray-500">{formatFileSize(share.file_size)}</p>
                  )}
                </div>
              </div>

              {share?.expires_at && (
                <p className="text-xs text-gray-500 mb-4">
                  过期时间：{new Date(share.expires_at).toLocaleString()}
                </p>
              )}

              <button
                onClick={handleDownload}
                className="w-full flex items-center justify-center gap-2 py-2.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                <Download className="w-4 h-4" />
                下载文件
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
