import { useState } from 'react'
import { useNavigate, Link } from 'react-router-dom'
import { useAuthStore } from '../store/auth'
import { HardDrive, Cloud, Shield, Zap } from 'lucide-react'

export default function RegisterPage() {
  const navigate = useNavigate()
  const register = useAuthStore((s) => s.register)
  const [username, setUsername] = useState('')
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [confirmPassword, setConfirmPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (password !== confirmPassword) {
      setError('两次密码输入不一致')
      return
    }

    setLoading(true)
    try {
      await register(username, email, password)
      navigate('/files')
    } catch (err: unknown) {
      setError(err instanceof Error ? err.message : '注册失败')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex bg-gradient-to-br from-[#f0f4f8] to-[#e8f0fa]">
      {/* Decorative circles */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div className="animate-float absolute -top-20 -right-20 w-72 h-72 rounded-full bg-[#d4e8fd]/30" />
        <div className="animate-float-delayed absolute bottom-1/4 -left-16 w-56 h-56 rounded-full bg-[#b3d4fc]/20" />
        <div className="animate-float-slow absolute top-10 left-1/3 w-40 h-40 rounded-full bg-[#e8f4fd]/40" />
      </div>

      {/* Left decorative panel */}
      <div className="hidden lg:flex flex-1 items-center justify-center relative">
        <div className="text-center max-w-md px-8">
          <div className="flex items-center justify-center mb-8">
            <div className="w-20 h-20 rounded-2xl bg-white/80 backdrop-blur-sm shadow-lg flex items-center justify-center">
              <HardDrive className="w-10 h-10 text-[#5b8db8]" />
            </div>
          </div>
          <h1 className="text-3xl font-bold text-[#2c3e50] mb-3">MediaDrive Pro</h1>
          <p className="text-[#6b7c93] text-lg mb-10">加入我们，开始你的云端之旅</p>

          <div className="space-y-4 text-left">
            <div className="flex items-center gap-3 bg-white/60 backdrop-blur-sm rounded-xl p-4">
              <Cloud className="w-6 h-6 text-[#5b8db8] shrink-0" />
              <div>
                <p className="font-medium text-[#2c3e50]">云端存储</p>
                <p className="text-sm text-[#6b7c93]">随时随地访问你的文件</p>
              </div>
            </div>
            <div className="flex items-center gap-3 bg-white/60 backdrop-blur-sm rounded-xl p-4">
              <Shield className="w-6 h-6 text-[#5b8db8] shrink-0" />
              <div>
                <p className="font-medium text-[#2c3e50]">安全可靠</p>
                <p className="text-sm text-[#6b7c93]">端到端加密保护你的数据</p>
              </div>
            </div>
            <div className="flex items-center gap-3 bg-white/60 backdrop-blur-sm rounded-xl p-4">
              <Zap className="w-6 h-6 text-[#5b8db8] shrink-0" />
              <div>
                <p className="font-medium text-[#2c3e50]">极速传输</p>
                <p className="text-sm text-[#6b7c93]">分片上传，大文件也能轻松搞定</p>
              </div>
            </div>
          </div>
        </div>
      </div>

      {/* Right form panel */}
      <div className="flex-1 flex items-center justify-center p-6 relative">
        <div className="w-full max-w-[420px] animate-fade-in-up">
          {/* Mobile logo */}
          <div className="flex items-center justify-center gap-2 mb-8 lg:hidden">
            <HardDrive className="w-8 h-8 text-[#5b8db8]" />
            <h1 className="text-2xl font-bold text-[#2c3e50]">MediaDrive Pro</h1>
          </div>

          <div className="bg-white/80 backdrop-blur-sm rounded-[20px] shadow-[0_8px_32px_rgba(0,0,0,0.08)] p-8">
            <h2 className="text-2xl font-semibold text-[#2c3e50] mb-1">创建账号</h2>
            <p className="text-[#6b7c93] text-sm mb-6">注册一个新的 MediaDrive Pro 账号</p>

            {error && (
              <div className="text-sm text-red-600 bg-red-50 p-3 rounded-xl mb-4">{error}</div>
            )}

            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-[#4a5568] mb-1.5">用户名</label>
                <input
                  type="text"
                  value={username}
                  onChange={(e) => setUsername(e.target.value)}
                  placeholder="请输入用户名"
                  className="w-full px-4 py-3 bg-[#f5f5f7] border-0 rounded-xl text-[#2c3e50] placeholder:text-[#a0aec0] focus:outline-none focus:shadow-[0_0_0_3px_rgba(179,212,252,0.3)] transition-shadow"
                  required
                  autoFocus
                  minLength={2}
                  maxLength={64}
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-[#4a5568] mb-1.5">邮箱</label>
                <input
                  type="email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  placeholder="请输入邮箱地址"
                  className="w-full px-4 py-3 bg-[#f5f5f7] border-0 rounded-xl text-[#2c3e50] placeholder:text-[#a0aec0] focus:outline-none focus:shadow-[0_0_0_3px_rgba(179,212,252,0.3)] transition-shadow"
                  required
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-[#4a5568] mb-1.5">密码</label>
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  placeholder="请输入密码（至少6位）"
                  className="w-full px-4 py-3 bg-[#f5f5f7] border-0 rounded-xl text-[#2c3e50] placeholder:text-[#a0aec0] focus:outline-none focus:shadow-[0_0_0_3px_rgba(179,212,252,0.3)] transition-shadow"
                  required
                  minLength={6}
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-[#4a5568] mb-1.5">确认密码</label>
                <input
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  placeholder="请再次输入密码"
                  className="w-full px-4 py-3 bg-[#f5f5f7] border-0 rounded-xl text-[#2c3e50] placeholder:text-[#a0aec0] focus:outline-none focus:shadow-[0_0_0_3px_rgba(179,212,252,0.3)] transition-shadow"
                  required
                  minLength={6}
                />
              </div>

              <button
                type="submit"
                disabled={loading}
                className="w-full py-3 bg-[#e8f4fd] text-[#5b8db8] border border-[#b3d4fc] rounded-xl font-medium hover:bg-[#d4e8fd] disabled:opacity-50 transition-all duration-200 active:scale-[0.98]"
              >
                {loading ? '注册中...' : '注册'}
              </button>
            </form>

            <p className="text-center text-sm text-[#6b7c93] mt-6">
              已有账号？{' '}
              <Link to="/login" className="text-[#5b8db8] font-medium hover:text-[#4a7da8] transition-colors">登录</Link>
            </p>
          </div>
        </div>
      </div>
    </div>
  )
}
