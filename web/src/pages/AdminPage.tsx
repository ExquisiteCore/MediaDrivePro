import { useState, useEffect } from 'react'
import { listUsers } from '../api/admin'
import type { User } from '../api/auth'
import { formatFileSize, formatDateTime } from '../lib/format'
import { useAuthStore } from '../store/auth'
import { Navigate } from 'react-router-dom'

export default function AdminPage() {
  const currentUser = useAuthStore((s) => s.user)
  const [users, setUsers] = useState<User[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')

  useEffect(() => {
    listUsers()
      .then(setUsers)
      .catch((err) => setError(err instanceof Error ? err.message : '加载失败'))
      .finally(() => setLoading(false))
  }, [])

  if (currentUser?.role !== 'admin') {
    return <Navigate to="/files" replace />
  }

  if (loading) {
    return <div className="p-8 text-center text-gray-400">加载中...</div>
  }

  return (
    <div className="p-6">
      <h2 className="text-xl font-semibold text-gray-900 mb-4">用户管理</h2>

      {error && <div className="text-sm text-red-600 bg-red-50 p-3 rounded-lg mb-4">{error}</div>}

      <div className="bg-white rounded-lg border border-gray-200 overflow-hidden">
        <table className="w-full">
          <thead className="bg-gray-50 border-b border-gray-200">
            <tr>
              <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">用户名</th>
              <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">邮箱</th>
              <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">角色</th>
              <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">存储使用</th>
              <th className="px-4 py-2 text-left text-xs font-medium text-gray-500 uppercase">注册时间</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-gray-100">
            {users.map(user => {
              const percent = user.storage_quota > 0 ? (user.storage_used / user.storage_quota) * 100 : 0
              return (
                <tr key={user.id} className="hover:bg-gray-50">
                  <td className="px-4 py-2.5 text-sm text-gray-900 font-medium">{user.username}</td>
                  <td className="px-4 py-2.5 text-sm text-gray-500">{user.email}</td>
                  <td className="px-4 py-2.5">
                    <span className={`text-xs px-2 py-0.5 rounded ${user.role === 'admin' ? 'bg-purple-100 text-purple-700' : 'bg-gray-100 text-gray-700'}`}>
                      {user.role}
                    </span>
                  </td>
                  <td className="px-4 py-2.5">
                    <div className="flex items-center gap-2">
                      <div className="w-24 bg-gray-200 rounded-full h-1.5">
                        <div
                          className={`h-1.5 rounded-full ${percent > 90 ? 'bg-red-500' : percent > 70 ? 'bg-yellow-500' : 'bg-blue-500'}`}
                          style={{ width: `${Math.min(percent, 100)}%` }}
                        />
                      </div>
                      <span className="text-xs text-gray-500">{formatFileSize(user.storage_used)} / {formatFileSize(user.storage_quota)}</span>
                    </div>
                  </td>
                  <td className="px-4 py-2.5 text-sm text-gray-500">{formatDateTime(user.created_at)}</td>
                </tr>
              )
            })}
          </tbody>
        </table>
      </div>
    </div>
  )
}
