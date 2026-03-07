import { useState, useEffect } from 'react'
import { useNavigate } from 'react-router-dom'
import { Plus, LogIn, Copy, Users, Tv2, X } from 'lucide-react'
import * as roomsApi from '../api/rooms'
import type { Room } from '../api/rooms'

export default function RoomsPage() {
  const [rooms, setRooms] = useState<Room[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreate, setShowCreate] = useState(false)
  const [showJoin, setShowJoin] = useState(false)
  const [createName, setCreateName] = useState('')
  const [createMax, setCreateMax] = useState(20)
  const [joinCode, setJoinCode] = useState('')
  const [error, setError] = useState('')
  const navigate = useNavigate()

  const fetchRooms = async () => {
    try {
      setLoading(true)
      const data = await roomsApi.listRooms()
      setRooms(data)
    } catch (e: any) {
      setError(e.message)
    } finally {
      setLoading(false)
    }
  }

  useEffect(() => {
    fetchRooms()
  }, [])

  const handleCreate = async () => {
    if (!createName.trim()) return
    try {
      const room = await roomsApi.createRoom(createName.trim(), createMax)
      setShowCreate(false)
      setCreateName('')
      navigate(`/rooms/${room.id}`)
    } catch (e: any) {
      setError(e.message)
    }
  }

  const handleJoin = async () => {
    if (!joinCode.trim()) return
    try {
      const room = await roomsApi.joinRoom(joinCode.trim())
      setShowJoin(false)
      setJoinCode('')
      navigate(`/rooms/${room.id}`)
    } catch (e: any) {
      setError(e.message)
    }
  }

  const copyCode = (code: string) => {
    navigator.clipboard.writeText(code)
  }

  const statusBadge = (status: string) => {
    const map: Record<string, string> = {
      waiting: 'bg-yellow-100 text-yellow-700',
      playing: 'bg-green-100 text-green-700',
      paused: 'bg-gray-100 text-gray-600',
      ended: 'bg-red-100 text-red-700',
    }
    const label: Record<string, string> = {
      waiting: '等待中',
      playing: '播放中',
      paused: '已暂停',
      ended: '已结束',
    }
    return (
      <span
        className={`text-xs px-2 py-0.5 rounded-full ${map[status] || 'bg-gray-100 text-gray-600'}`}
      >
        {label[status] || status}
      </span>
    )
  }

  return (
    <div className="p-6 max-w-5xl mx-auto">
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-2">
          <Tv2 className="w-6 h-6 text-blue-600" />
          <h1 className="text-2xl font-bold text-gray-900">观影室</h1>
        </div>
        <div className="flex gap-2">
          <button
            onClick={() => setShowJoin(true)}
            className="flex items-center gap-1.5 px-4 py-2 text-sm bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
          >
            <LogIn className="w-4 h-4" />
            加入房间
          </button>
          <button
            onClick={() => setShowCreate(true)}
            className="flex items-center gap-1.5 px-4 py-2 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <Plus className="w-4 h-4" />
            创建房间
          </button>
        </div>
      </div>

      {error && (
        <div className="mb-4 p-3 bg-red-50 text-red-600 rounded-lg text-sm">
          {error}
        </div>
      )}

      {loading ? (
        <div className="text-center text-gray-400 py-12">加载中...</div>
      ) : rooms.length === 0 ? (
        <div className="text-center py-16">
          <Tv2 className="w-12 h-12 text-gray-300 mx-auto mb-3" />
          <p className="text-gray-500">还没有观影室，创建一个或输入邀请码加入</p>
        </div>
      ) : (
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {rooms.map((room) => (
            <div
              key={room.id}
              onClick={() => room.status !== 'ended' && navigate(`/rooms/${room.id}`)}
              className={`bg-white rounded-xl border border-gray-200 p-4 transition-shadow ${
                room.status !== 'ended'
                  ? 'hover:shadow-md cursor-pointer'
                  : 'opacity-60'
              }`}
            >
              <div className="flex items-start justify-between mb-3">
                <h3 className="font-semibold text-gray-900 truncate">{room.name}</h3>
                {statusBadge(room.status)}
              </div>
              <div className="flex items-center gap-3 text-xs text-gray-500">
                <span className="flex items-center gap-1">
                  <Users className="w-3.5 h-3.5" />
                  {room.member_count}/{room.max_members}
                </span>
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    copyCode(room.invite_code)
                  }}
                  className="flex items-center gap-1 hover:text-blue-600 transition-colors"
                  title="复制邀请码"
                >
                  <Copy className="w-3.5 h-3.5" />
                  {room.invite_code}
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Create modal */}
      {showCreate && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 w-full max-w-md shadow-xl">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">创建观影室</h2>
              <button onClick={() => setShowCreate(false)} className="text-gray-400 hover:text-gray-600">
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  房间名称
                </label>
                <input
                  type="text"
                  value={createName}
                  onChange={(e) => setCreateName(e.target.value)}
                  placeholder="例如：周末电影之夜"
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                  maxLength={128}
                  autoFocus
                  onKeyDown={(e) => e.key === 'Enter' && handleCreate()}
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  最大人数
                </label>
                <input
                  type="number"
                  value={createMax}
                  onChange={(e) => setCreateMax(parseInt(e.target.value) || 20)}
                  min={2}
                  max={100}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>
              <button
                onClick={handleCreate}
                className="w-full py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Join modal */}
      {showJoin && (
        <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
          <div className="bg-white rounded-xl p-6 w-full max-w-md shadow-xl">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">加入观影室</h2>
              <button onClick={() => setShowJoin(false)} className="text-gray-400 hover:text-gray-600">
                <X className="w-5 h-5" />
              </button>
            </div>
            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  邀请码
                </label>
                <input
                  type="text"
                  value={joinCode}
                  onChange={(e) => setJoinCode(e.target.value)}
                  placeholder="输入 8 位邀请码"
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                  maxLength={16}
                  autoFocus
                  onKeyDown={(e) => e.key === 'Enter' && handleJoin()}
                />
              </div>
              <button
                onClick={handleJoin}
                className="w-full py-2 bg-blue-600 text-white rounded-lg text-sm font-medium hover:bg-blue-700 transition-colors"
              >
                加入
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
