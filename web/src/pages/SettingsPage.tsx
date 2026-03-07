import { useRef, useState } from 'react'
import { useAuthStore } from '../store/auth'
import { uploadAvatar } from '../api/auth'
import { formatFileSize } from '../lib/format'
import { formatDateTime } from '../lib/format'
import { User, Mail, Shield, Calendar, Camera } from 'lucide-react'
import StorageBar from '../components/StorageBar'

export default function SettingsPage() {
  const { user, updateUser } = useAuthStore()
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [uploading, setUploading] = useState(false)
  const [previewUrl, setPreviewUrl] = useState<string | null>(null)

  if (!user) {
    return <div className="p-8 text-center text-gray-400">加载中...</div>
  }

  const handleAvatarChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (!file) return

    const reader = new FileReader()
    reader.onload = (ev) => setPreviewUrl(ev.target?.result as string)
    reader.readAsDataURL(file)

    setUploading(true)
    try {
      const updated = await uploadAvatar(file)
      updateUser(updated)
    } catch {
      // keep preview even if upload fails
    } finally {
      setUploading(false)
    }
  }

  const avatarSrc = previewUrl || (user.avatar ? `/api/v1/users/${user.id}/avatar` : null)

  return (
    <div className="p-6 max-w-2xl">
      <h2 className="text-xl font-semibold text-gray-900 mb-6">设置</h2>

      {/* Profile */}
      <div className="bg-white rounded-lg border border-gray-200 p-6 mb-6">
        <h3 className="text-lg font-medium text-gray-900 mb-4">个人信息</h3>

        {/* Avatar */}
        <div className="flex items-center gap-4 mb-6">
          <div
            className="relative w-16 h-16 rounded-full cursor-pointer group shrink-0"
            onClick={() => fileInputRef.current?.click()}
          >
            <div className="w-full h-full rounded-full bg-gradient-to-br from-[#b3d4fc] to-[#5b8db8] p-[2px]">
              <div className="w-full h-full rounded-full bg-white flex items-center justify-center overflow-hidden">
                {avatarSrc ? (
                  <img src={avatarSrc} alt="avatar" className="w-full h-full object-cover" />
                ) : (
                  <span className="text-2xl font-bold text-[#5b8db8]/40">
                    {user.username.charAt(0).toUpperCase()}
                  </span>
                )}
              </div>
            </div>
            <div className="absolute inset-0 rounded-full bg-black/0 group-hover:bg-black/20 transition-colors flex items-center justify-center">
              <Camera className="w-4 h-4 text-white opacity-0 group-hover:opacity-100 transition-opacity" />
            </div>
            {uploading && (
              <div className="absolute inset-0 rounded-full bg-white/60 flex items-center justify-center">
                <div className="w-6 h-6 border-2 border-[#5b8db8] border-t-transparent rounded-full animate-spin" />
              </div>
            )}
          </div>
          <div>
            <p className="font-medium text-gray-900">{user.username}</p>
            <button
              onClick={() => fileInputRef.current?.click()}
              className="text-sm text-[#5b8db8] hover:text-[#4a7da8] transition-colors"
            >
              更换头像
            </button>
          </div>
          <input
            ref={fileInputRef}
            type="file"
            accept="image/*"
            onChange={handleAvatarChange}
            className="hidden"
          />
        </div>

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
