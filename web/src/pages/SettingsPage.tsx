import { useAuthStore } from '../store/auth'
import { formatFileSize } from '../lib/format'
import { formatDateTime } from '../lib/format'
import { User, Mail, Shield, Calendar } from 'lucide-react'
import StorageBar from '../components/StorageBar'

export default function SettingsPage() {
  const user = useAuthStore((s) => s.user)

  if (!user) {
    return <div className="p-8 text-center text-gray-400">加载中...</div>
  }

  return (
    <div className="p-6 max-w-2xl">
      <h2 className="text-xl font-semibold text-gray-900 mb-6">设置</h2>

      {/* Profile */}
      <div className="bg-white rounded-lg border border-gray-200 p-6 mb-6">
        <h3 className="text-lg font-medium text-gray-900 mb-4">个人信息</h3>
        <div className="space-y-3">
          <div className="flex items-center gap-3">
            <User className="w-4 h-4 text-gray-400" />
            <span className="text-sm text-gray-500 w-16">用户名</span>
            <span className="text-sm text-gray-900">{user.username}</span>
          </div>
          <div className="flex items-center gap-3">
            <Mail className="w-4 h-4 text-gray-400" />
            <span className="text-sm text-gray-500 w-16">邮箱</span>
            <span className="text-sm text-gray-900">{user.email}</span>
          </div>
          <div className="flex items-center gap-3">
            <Shield className="w-4 h-4 text-gray-400" />
            <span className="text-sm text-gray-500 w-16">角色</span>
            <span className={`text-sm px-2 py-0.5 rounded ${user.role === 'admin' ? 'bg-purple-100 text-purple-700' : 'bg-gray-100 text-gray-700'}`}>
              {user.role}
            </span>
          </div>
          <div className="flex items-center gap-3">
            <Calendar className="w-4 h-4 text-gray-400" />
            <span className="text-sm text-gray-500 w-16">注册于</span>
            <span className="text-sm text-gray-900">{formatDateTime(user.created_at)}</span>
          </div>
        </div>
      </div>

      {/* Storage */}
      <div className="bg-white rounded-lg border border-gray-200 p-6">
        <h3 className="text-lg font-medium text-gray-900 mb-4">存储空间</h3>
        <StorageBar used={user.storage_used} quota={user.storage_quota} />
        <div className="mt-3 text-sm text-gray-500">
          已使用 {formatFileSize(user.storage_used)}，共 {formatFileSize(user.storage_quota)}
        </div>
      </div>
    </div>
  )
}
