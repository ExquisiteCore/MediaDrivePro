import { useState, useEffect, useCallback } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { Copy, LogOut, Users, Film, X, Wifi, WifiOff } from 'lucide-react'
import * as roomsApi from '../api/rooms'
import { listFiles } from '../api/files'
import { listTranscodes } from '../api/transcode'
import { useAuthStore } from '../store/auth'
import { useRoomSocket } from '../hooks/useRoomSocket'
import RoomPlayer from '../components/RoomPlayer'
import ChatPanel from '../components/ChatPanel'
import type { Room, RoomMember } from '../api/rooms'
import type { FileInfo } from '../api/files'

export default function WatchRoomPage() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const { user } = useAuthStore()
  const [room, setRoom] = useState<Room | null>(null)
  const [apiMembers, setApiMembers] = useState<RoomMember[]>([])
  const [showFilePicker, setShowFilePicker] = useState(false)
  const [error, setError] = useState('')
  const [manualFileId, setManualFileId] = useState<string | null>(null)

  const {
    connected,
    playState,
    messages,
    danmakuList,
    members: wsMembers,
    clockOffset,
    sendChat,
    sendDanmaku,
    sendPlay,
    sendPause,
    sendSeek,
    removeDanmaku,
  } = useRoomSocket(id)

  const activeFileId = playState.fileId || manualFileId
  const isHost = room?.host_id === user?.id

  // Fetch room detail
  useEffect(() => {
    if (!id) return
    roomsApi.getRoom(id).then((detail) => {
      setRoom(detail.room)
      setApiMembers(detail.members)
    }).catch((e: unknown) => {
      setError(e instanceof Error ? e.message : '请求失败')
    })
  }, [id])

  const handleClose = async () => {
    if (!id || !isHost) return
    if (!confirm('确定关闭房间？所有成员将被断开连接。')) return
    try {
      await roomsApi.closeRoom(id)
      navigate('/rooms')
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : '请求失败')
    }
  }

  const handleLeave = () => {
    navigate('/rooms')
  }

  const handleSelectFile = async (fileId: string) => {
    if (!id) return
    try {
      await roomsApi.setPlaying(id, fileId)
      setManualFileId(fileId)
      setShowFilePicker(false)
    } catch (e: unknown) {
      setError(e instanceof Error ? e.message : '请求失败')
    }
  }

  const copyCode = () => {
    if (room) {
      navigator.clipboard.writeText(room.invite_code)
    }
  }

  if (error && !room) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center">
          <p className="text-red-500 mb-4">{error}</p>
          <button
            onClick={() => navigate('/rooms')}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg text-sm"
          >
            返回列表
          </button>
        </div>
      </div>
    )
  }

  if (!room) {
    return (
      <div className="flex items-center justify-center h-full text-gray-400">
        加载中...
      </div>
    )
  }

  return (
    <div className="flex flex-col h-full">
      {/* Top bar */}
      <div className="flex items-center justify-between px-4 py-2 bg-white border-b border-gray-200">
        <div className="flex items-center gap-3">
          <h2 className="font-semibold text-gray-900">{room.name}</h2>
          <button
            onClick={copyCode}
            className="flex items-center gap-1 text-xs text-gray-500 hover:text-blue-600 bg-gray-100 px-2 py-1 rounded"
            title="复制邀请码"
          >
            <Copy className="w-3 h-3" />
            {room.invite_code}
          </button>
          {connected ? (
            <span className="flex items-center gap-1 text-xs text-green-600">
              <Wifi className="w-3 h-3" /> 已连接
            </span>
          ) : (
            <span className="flex items-center gap-1 text-xs text-red-500">
              <WifiOff className="w-3 h-3" /> 断开
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {isHost && (
            <button
              onClick={() => setShowFilePicker(true)}
              className="flex items-center gap-1 text-sm px-3 py-1.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
            >
              <Film className="w-4 h-4" />
              选择视频
            </button>
          )}
          {isHost ? (
            <button
              onClick={handleClose}
              className="text-sm px-3 py-1.5 text-red-600 border border-red-200 rounded-lg hover:bg-red-50"
            >
              关闭房间
            </button>
          ) : (
            <button
              onClick={handleLeave}
              className="flex items-center gap-1 text-sm px-3 py-1.5 text-gray-600 border border-gray-200 rounded-lg hover:bg-gray-50"
            >
              <LogOut className="w-4 h-4" />
              离开
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="px-4 py-2 bg-red-50 text-red-600 text-sm">{error}</div>
      )}

      {/* Main content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Video area */}
        <div className="flex-1 p-4 flex flex-col">
          <RoomPlayer
            fileId={activeFileId}
            playState={playState}
            clockOffset={clockOffset}
            isHost={isHost}
            danmakuList={danmakuList}
            onRemoveDanmaku={removeDanmaku}
            onPlay={sendPlay}
            onPause={sendPause}
            onSeek={sendSeek}
          />
        </div>

        {/* Right panel */}
        <div className="w-80 border-l border-gray-200 flex flex-col bg-white">
          {/* Members */}
          <div className="p-3 border-b border-gray-200">
            <div className="flex items-center gap-1.5 text-sm font-medium text-gray-700 mb-2">
              <Users className="w-4 h-4" />
              成员 ({wsMembers.length || apiMembers.length})
            </div>
            <div className="flex flex-wrap gap-1.5">
              {(wsMembers.length > 0 ? wsMembers : apiMembers).map((m) => {
                const name = 'name' in m ? m.name : ('username' in m ? (m as RoomMember).username : '')
                const memberId = 'id' in m ? m.id : ('user_id' in m ? (m as RoomMember).user_id : '')
                return (
                  <span
                    key={memberId}
                    className={`text-xs px-2 py-1 rounded-full ${
                      memberId === room.host_id
                        ? 'bg-blue-100 text-blue-700'
                        : 'bg-gray-100 text-gray-600'
                    }`}
                  >
                    {name}
                    {memberId === room.host_id && ' (房主)'}
                  </span>
                )
              })}
            </div>
          </div>

          {/* Chat */}
          <div className="flex-1 overflow-hidden">
            <ChatPanel
              messages={messages}
              onSendChat={sendChat}
              onSendDanmaku={sendDanmaku}
            />
          </div>
        </div>
      </div>

      {/* File picker modal */}
      {showFilePicker && (
        <FilePickerModal
          onSelect={handleSelectFile}
          onClose={() => setShowFilePicker(false)}
        />
      )}
    </div>
  )
}

function FilePickerModal({
  onSelect,
  onClose,
}: {
  onSelect: (fileId: string) => void
  onClose: () => void
}) {
  const [files, setFiles] = useState<FileInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [transcodedFiles, setTranscodedFiles] = useState<Set<string>>(new Set())

  const loadFiles = useCallback(async () => {
    try {
      setLoading(true)
      const res = await listFiles({ per_page: 100 })
      // Filter to video files
      const videoFiles = res.data.filter((f) =>
        f.content_type.startsWith('video/'),
      )
      setFiles(videoFiles)

      // Check transcode status for each
      const transcoded = new Set<string>()
      await Promise.all(
        videoFiles.map(async (f) => {
          try {
            const tasks = await listTranscodes(f.id)
            if (tasks.some((t) => t.status === 'completed')) {
              transcoded.add(f.id)
            }
          } catch {
            // ignore
          }
        }),
      )
      setTranscodedFiles(transcoded)
    } catch {
      // ignore
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    loadFiles()
  }, [loadFiles])

  return (
    <div className="fixed inset-0 bg-black/40 flex items-center justify-center z-50">
      <div className="bg-white rounded-xl p-6 w-full max-w-lg shadow-xl max-h-[70vh] flex flex-col">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold">选择视频</h2>
          <button onClick={onClose} className="text-gray-400 hover:text-gray-600">
            <X className="w-5 h-5" />
          </button>
        </div>

        <p className="text-xs text-gray-500 mb-3">
          只有已完成转码的视频文件才能在观影室播放
        </p>

        <div className="flex-1 overflow-y-auto space-y-1">
          {loading ? (
            <div className="text-center text-gray-400 py-8">加载中...</div>
          ) : files.length === 0 ? (
            <div className="text-center text-gray-400 py-8">
              没有找到视频文件
            </div>
          ) : (
            files.map((f) => {
              const ready = transcodedFiles.has(f.id)
              return (
                <button
                  key={f.id}
                  onClick={() => ready && onSelect(f.id)}
                  disabled={!ready}
                  className={`w-full text-left px-3 py-2.5 rounded-lg flex items-center justify-between ${
                    ready
                      ? 'hover:bg-blue-50 cursor-pointer'
                      : 'opacity-50 cursor-not-allowed'
                  }`}
                >
                  <div className="flex items-center gap-2 min-w-0">
                    <Film className="w-4 h-4 text-gray-400 shrink-0" />
                    <span className="text-sm truncate">{f.name}</span>
                  </div>
                  <span
                    className={`text-xs px-2 py-0.5 rounded-full shrink-0 ml-2 ${
                      ready
                        ? 'bg-green-100 text-green-700'
                        : 'bg-gray-100 text-gray-500'
                    }`}
                  >
                    {ready ? '可播放' : '未转码'}
                  </span>
                </button>
              )
            })
          )}
        </div>
      </div>
    </div>
  )
}
