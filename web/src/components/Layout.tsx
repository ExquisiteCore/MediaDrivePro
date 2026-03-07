import { NavLink, Outlet, useNavigate } from 'react-router-dom'
import { useAuthStore } from '../store/auth'
import {
  FolderOpen,
  Share2,
  Key,
  Settings,
  LogOut,
  Shield,
  HardDrive,
} from 'lucide-react'
import StorageBar from './StorageBar'

export default function Layout() {
  const { user, logout } = useAuthStore()
  const navigate = useNavigate()

  const handleLogout = () => {
    logout()
    navigate('/login')
  }

  const linkClass = ({ isActive }: { isActive: boolean }) =>
    `flex items-center gap-3 px-3 py-2 rounded-lg text-sm transition-colors ${
      isActive
        ? 'bg-blue-50 text-blue-700 font-medium'
        : 'text-gray-600 hover:bg-gray-100 hover:text-gray-900'
    }`

  return (
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <aside className="w-60 bg-white border-r border-gray-200 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <div className="flex items-center gap-2">
            <HardDrive className="w-6 h-6 text-blue-600" />
            <h1 className="text-lg font-bold text-gray-900">MediaDrive</h1>
          </div>
        </div>

        <nav className="flex-1 p-3 space-y-1">
          <NavLink to="/files" className={linkClass} end={false}>
            <FolderOpen className="w-4 h-4" />
            文件
          </NavLink>
          <NavLink to="/shares" className={linkClass}>
            <Share2 className="w-4 h-4" />
            分享管理
          </NavLink>
          <NavLink to="/tokens" className={linkClass}>
            <Key className="w-4 h-4" />
            API Token
          </NavLink>
          <NavLink to="/settings" className={linkClass}>
            <Settings className="w-4 h-4" />
            设置
          </NavLink>
          {user?.role === 'admin' && (
            <NavLink to="/admin" className={linkClass}>
              <Shield className="w-4 h-4" />
              管理
            </NavLink>
          )}
        </nav>

        {/* Storage bar */}
        {user && (
          <div className="p-3 border-t border-gray-200">
            <StorageBar used={user.storage_used} quota={user.storage_quota} />
          </div>
        )}

        {/* User info & logout */}
        <div className="p-3 border-t border-gray-200">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2 min-w-0">
              <div className="w-8 h-8 rounded-full bg-gradient-to-br from-[#b3d4fc] to-[#5b8db8] p-[1.5px] shrink-0">
                <div className="w-full h-full rounded-full bg-white flex items-center justify-center overflow-hidden">
                  {user?.avatar ? (
                    <img src={`/api/v1/users/${user.id}/avatar`} alt="" className="w-full h-full object-cover" />
                  ) : (
                    <span className="text-xs font-bold text-[#5b8db8]/60">
                      {user?.username?.charAt(0).toUpperCase()}
                    </span>
                  )}
                </div>
              </div>
              <span className="text-sm text-gray-700 truncate">{user?.username}</span>
            </div>
            <button
              onClick={handleLogout}
              className="p-1.5 text-gray-400 hover:text-red-500 rounded transition-colors"
              title="退出登录"
            >
              <LogOut className="w-4 h-4" />
            </button>
          </div>
        </div>
      </aside>

      {/* Main content */}
      <main className="flex-1 overflow-auto">
        <Outlet />
      </main>
    </div>
  )
}
