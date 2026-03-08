import { useEffect, useRef, useState, useCallback } from 'react'

export interface UserBrief {
  id: string
  name: string
  avatar: string | null
}

export interface ChatMessage {
  type: 'chat' | 'system'
  user?: UserBrief
  content: string
  timestamp: number
}

export interface DanmakuItem {
  id: string
  user_id: string
  content: string
  color: string
  position: string
  video_time: number
}

export interface PlayState {
  status: string
  time: number
  playbackRate: number
  fileId: string | null
  serverTime: number
}

export interface VideoState {
  paused: boolean
  currentTime: number
  playbackRate: number
}

// --- Server → Client message types ---

interface WsOutSync {
  type: 'sync'
  status: string
  time: number
  playback_rate: number
  file_id: string | null
  server_time: number
}

interface WsOutCheckStatus {
  type: 'check_status'
  is_playing: boolean
  current_time: number
  playback_rate: number
}

interface WsOutViewerCount {
  type: 'viewer_count'
  count: number
}

interface WsOutMemberJoin {
  type: 'member_join'
  user: UserBrief
}

interface WsOutMemberLeave {
  type: 'member_leave'
  user_id: string
}

interface WsOutChat {
  type: 'chat'
  user: UserBrief
  content: string
}

interface WsOutDanmaku {
  type: 'danmaku'
  user_id: string
  content: string
  color: string
  position: string
  video_time: number
}

interface WsOutPong {
  type: 'pong'
  server_time: number
}

interface WsOutError {
  type: 'error'
  code: string
  message: string
}

type WsOutMessage =
  | WsOutSync
  | WsOutCheckStatus
  | WsOutViewerCount
  | WsOutMemberJoin
  | WsOutMemberLeave
  | WsOutChat
  | WsOutDanmaku
  | WsOutPong
  | WsOutError

export function useRoomSocket(
  roomId: string | undefined,
  getVideoState?: () => VideoState | null,
) {
  const wsRef = useRef<WebSocket | null>(null)
  const [connected, setConnected] = useState(false)
  const [playState, setPlayState] = useState<PlayState>({
    status: 'waiting',
    time: 0,
    playbackRate: 1.0,
    fileId: null,
    serverTime: 0,
  })
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [danmakuList, setDanmakuList] = useState<DanmakuItem[]>([])
  const [members, setMembers] = useState<UserBrief[]>([])
  const [viewerCount, setViewerCount] = useState(0)
  const [clockOffset, setClockOffset] = useState(0)
  const retryRef = useRef(0)
  const retryTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const pingTimerRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const checkStatusTimerRef = useRef<ReturnType<typeof setInterval> | null>(null)
  const danmakuIdRef = useRef(0)
  const connectRef = useRef<(() => void) | null>(null)
  const pingStartRef = useRef(0)
  const getVideoStateRef = useRef(getVideoState)

  // Keep the ref up to date
  useEffect(() => {
    getVideoStateRef.current = getVideoState
  }, [getVideoState])

  const connect = useCallback(() => {
    if (!roomId) return

    const token = localStorage.getItem('token')
    if (!token) return

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const url = `${protocol}//${window.location.host}/api/v1/rooms/${roomId}/ws?token=${encodeURIComponent(token)}`

    const ws = new WebSocket(url)
    wsRef.current = ws

    ws.onopen = () => {
      setConnected(true)
      retryRef.current = 0

      // Ping interval for clock sync (every 5s)
      pingTimerRef.current = setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
          pingStartRef.current = Date.now()
          ws.send(JSON.stringify({ type: 'ping' }))
        }
      }, 5000)

      // CHECK_STATUS interval (every 10s)
      checkStatusTimerRef.current = setInterval(() => {
        if (ws.readyState !== WebSocket.OPEN) return
        const videoState = getVideoStateRef.current?.()
        if (videoState) {
          ws.send(
            JSON.stringify({
              type: 'check_status',
              is_playing: !videoState.paused,
              current_time: videoState.currentTime,
              playback_rate: videoState.playbackRate,
              timestamp: Date.now(),
            }),
          )
        }
      }, 10000)
    }

    ws.onmessage = (event) => {
      let msg: WsOutMessage
      try {
        msg = JSON.parse(event.data)
      } catch {
        return
      }

      switch (msg.type) {
        case 'sync':
          setPlayState({
            status: msg.status,
            time: msg.time,
            playbackRate: msg.playback_rate,
            fileId: msg.file_id,
            serverTime: msg.server_time,
          })
          break

        case 'check_status':
          // Server correction — force update playback state
          setPlayState((prev) => ({
            ...prev,
            status: msg.is_playing ? 'playing' : 'paused',
            time: msg.current_time,
            playbackRate: msg.playback_rate,
            serverTime: Date.now() / 1000,
          }))
          break

        case 'viewer_count':
          setViewerCount(msg.count)
          break

        case 'member_join':
          setMembers((prev) => {
            if (prev.some((m) => m.id === msg.user.id)) return prev
            return [...prev, msg.user]
          })
          setMessages((prev) => [
            ...prev,
            {
              type: 'system',
              content: `${msg.user.name} 加入了房间`,
              timestamp: Date.now(),
            },
          ])
          break

        case 'member_leave':
          setMembers((prev) => prev.filter((m) => m.id !== msg.user_id))
          setMessages((prev) => [
            ...prev,
            {
              type: 'system',
              content: `用户离开了房间`,
              timestamp: Date.now(),
            },
          ])
          break

        case 'chat':
          setMessages((prev) => [
            ...prev,
            {
              type: 'chat',
              user: msg.user,
              content: msg.content,
              timestamp: Date.now(),
            },
          ])
          break

        case 'danmaku':
          setDanmakuList((prev) => [
            ...prev,
            {
              id: `dm-${++danmakuIdRef.current}`,
              user_id: msg.user_id,
              content: msg.content,
              color: msg.color,
              position: msg.position,
              video_time: msg.video_time,
            },
          ])
          break

        case 'pong': {
          // RTT-aware clock offset
          const rtt = (Date.now() - pingStartRef.current) / 1000
          const offset = msg.server_time - Date.now() / 1000 + rtt / 2
          setClockOffset(offset)
          break
        }

        case 'error':
          setMessages((prev) => [
            ...prev,
            {
              type: 'system',
              content: `错误: ${msg.message}`,
              timestamp: Date.now(),
            },
          ])
          if (msg.code === 'ROOM_CLOSED') {
            ws.close()
          }
          break
      }
    }

    ws.onclose = () => {
      setConnected(false)
      if (pingTimerRef.current) clearInterval(pingTimerRef.current)
      if (checkStatusTimerRef.current) clearInterval(checkStatusTimerRef.current)

      // Reconnect with exponential backoff
      const delay = Math.min(1000 * Math.pow(2, retryRef.current), 16000)
      retryRef.current++
      retryTimerRef.current = setTimeout(() => connectRef.current?.(), delay)
    }

    ws.onerror = () => {
      ws.close()
    }
  }, [roomId])

  useEffect(() => {
    connectRef.current = connect
  }, [connect])

  useEffect(() => {
    connect()
    return () => {
      if (retryTimerRef.current) clearTimeout(retryTimerRef.current)
      if (pingTimerRef.current) clearInterval(pingTimerRef.current)
      if (checkStatusTimerRef.current) clearInterval(checkStatusTimerRef.current)
      retryRef.current = Infinity // prevent reconnect on unmount
      wsRef.current?.close()
    }
  }, [connect])

  // All outgoing messages include timestamp for latency compensation
  const send = useCallback((data: object) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({ ...data, timestamp: Date.now() }))
    }
  }, [])

  const sendChat = useCallback(
    (content: string) => send({ type: 'chat', content }),
    [send],
  )

  const sendDanmaku = useCallback(
    (content: string, color?: string, position?: string) =>
      send({ type: 'danmaku', content, color, position }),
    [send],
  )

  const sendPlay = useCallback(() => send({ type: 'play' }), [send])
  const sendPause = useCallback(() => send({ type: 'pause' }), [send])
  const sendSeek = useCallback(
    (time: number) => send({ type: 'seek', time }),
    [send],
  )

  const removeDanmaku = useCallback((id: string) => {
    setDanmakuList((prev) => prev.filter((d) => d.id !== id))
  }, [])

  return {
    connected,
    playState,
    messages,
    danmakuList,
    members,
    viewerCount,
    clockOffset,
    sendChat,
    sendDanmaku,
    sendPlay,
    sendPause,
    sendSeek,
    removeDanmaku,
  }
}
