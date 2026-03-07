import { useEffect, useRef, useCallback } from 'react'
import Hls from 'hls.js'
import {
  Play,
  Pause,
  Volume2,
  VolumeX,
  Maximize,
} from 'lucide-react'
import DanmakuOverlay from './DanmakuOverlay'
import type { PlayState, DanmakuItem } from '../hooks/useRoomSocket'

interface RoomPlayerProps {
  fileId: string | null
  playState: PlayState
  clockOffset: number
  isHost: boolean
  danmakuList: DanmakuItem[]
  onRemoveDanmaku: (id: string) => void
  onPlay: () => void
  onPause: () => void
  onSeek: (time: number) => void
}

export default function RoomPlayer({
  fileId,
  playState,
  clockOffset,
  isHost,
  danmakuList,
  onRemoveDanmaku,
  onPlay,
  onPause,
  onSeek,
}: RoomPlayerProps) {
  const videoRef = useRef<HTMLVideoElement>(null)
  const hlsRef = useRef<Hls | null>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const seekBarRef = useRef<HTMLInputElement>(null)
  const lastSyncRef = useRef(0)

  // Load HLS source
  useEffect(() => {
    const video = videoRef.current
    if (!video || !fileId) return

    const token = localStorage.getItem('token')
    const src = `/api/v1/stream/${fileId}/master.m3u8`

    if (Hls.isSupported()) {
      const hls = new Hls({
        xhrSetup: (xhr) => {
          if (token) {
            xhr.setRequestHeader('Authorization', `Bearer ${token}`)
          }
        },
      })
      hls.loadSource(src)
      hls.attachMedia(video)
      hlsRef.current = hls

      return () => {
        hls.destroy()
        hlsRef.current = null
      }
    } else if (video.canPlayType('application/vnd.apple.mpegurl')) {
      video.src = `${src}${src.includes('?') ? '&' : '?'}token=${encodeURIComponent(token || '')}`
    }
  }, [fileId])

  // Sync playback state
  useEffect(() => {
    const video = videoRef.current
    if (!video || !fileId) return

    const now = Date.now() / 1000
    const elapsed = now + clockOffset - playState.serverTime

    if (playState.status === 'playing') {
      const targetTime = playState.time + elapsed
      const diff = Math.abs(video.currentTime - targetTime)

      if (diff > 1.5) {
        video.currentTime = targetTime
      } else if (diff > 0.3) {
        video.playbackRate = video.currentTime < targetTime ? 1.05 : 0.95
      } else {
        video.playbackRate = 1.0
      }

      if (video.paused) {
        video.play().catch(() => {})
      }
    } else if (playState.status === 'paused') {
      if (!video.paused) {
        video.pause()
      }
      const diff = Math.abs(video.currentTime - playState.time)
      if (diff > 0.5) {
        video.currentTime = playState.time
      }
    } else if (playState.status === 'waiting') {
      video.pause()
      video.currentTime = 0
    }

    lastSyncRef.current = now
  }, [playState, fileId, clockOffset])

  // Update seek bar
  useEffect(() => {
    const video = videoRef.current
    if (!video) return

    const onTimeUpdate = () => {
      if (seekBarRef.current && video.duration) {
        seekBarRef.current.value = String(video.currentTime)
        seekBarRef.current.max = String(video.duration)
      }
    }
    video.addEventListener('timeupdate', onTimeUpdate)
    return () => video.removeEventListener('timeupdate', onTimeUpdate)
  }, [])

  const handleSeekBarChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (!isHost) return
      const time = parseFloat(e.target.value)
      onSeek(time)
    },
    [isHost, onSeek],
  )

  const toggleMute = useCallback(() => {
    const video = videoRef.current
    if (video) {
      video.muted = !video.muted
    }
  }, [])

  const toggleFullscreen = useCallback(() => {
    const el = containerRef.current
    if (!el) return
    if (document.fullscreenElement) {
      document.exitFullscreen()
    } else {
      el.requestFullscreen()
    }
  }, [])

  const formatTime = (s: number) => {
    if (!isFinite(s)) return '0:00'
    const m = Math.floor(s / 60)
    const sec = Math.floor(s % 60)
    return `${m}:${sec.toString().padStart(2, '0')}`
  }

  return (
    <div ref={containerRef} className="relative bg-black rounded-lg overflow-hidden">
      <video
        ref={videoRef}
        className="w-full aspect-video"
        crossOrigin="anonymous"
        playsInline
      />

      {/* Danmaku overlay */}
      <DanmakuOverlay items={danmakuList} onRemove={onRemoveDanmaku} />

      {/* No file placeholder */}
      {!fileId && (
        <div className="absolute inset-0 flex items-center justify-center text-gray-400 text-lg bg-gray-900">
          等待房主选择视频...
        </div>
      )}

      {/* Controls bar */}
      <div className="absolute bottom-0 left-0 right-0 bg-gradient-to-t from-black/80 to-transparent p-3">
        {/* Seek bar */}
        <input
          ref={seekBarRef}
          type="range"
          min="0"
          max="100"
          step="0.1"
          defaultValue="0"
          onChange={handleSeekBarChange}
          disabled={!isHost}
          className="w-full h-1 mb-2 accent-blue-500 cursor-pointer disabled:cursor-default disabled:opacity-50"
        />

        <div className="flex items-center gap-3 text-white text-sm">
          {/* Play/Pause */}
          <button
            onClick={playState.status === 'playing' ? onPause : onPlay}
            disabled={!isHost}
            className="hover:text-blue-400 disabled:opacity-50 disabled:cursor-default"
          >
            {playState.status === 'playing' ? (
              <Pause className="w-5 h-5" />
            ) : (
              <Play className="w-5 h-5" />
            )}
          </button>

          {/* Time */}
          <span className="text-xs tabular-nums">
            {formatTime(videoRef.current?.currentTime ?? 0)} /{' '}
            {formatTime(videoRef.current?.duration ?? 0)}
          </span>

          <div className="flex-1" />

          {/* Volume */}
          <button onClick={toggleMute} className="hover:text-blue-400">
            {videoRef.current?.muted ? (
              <VolumeX className="w-5 h-5" />
            ) : (
              <Volume2 className="w-5 h-5" />
            )}
          </button>

          {/* Fullscreen */}
          <button onClick={toggleFullscreen} className="hover:text-blue-400">
            <Maximize className="w-5 h-5" />
          </button>
        </div>
      </div>
    </div>
  )
}
