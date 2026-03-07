import { useState, useEffect, useCallback } from 'react'
import { X, Play, Loader2, AlertCircle, Film } from 'lucide-react'
import { getPreviewUrl } from '../api/files'
import { listTranscodes, createTranscode } from '../api/transcode'
import { getMediaInfo, scanMedia } from '../api/media'
import { getPreviewType, isTranscodableVideo } from '../lib/mime'
import { formatFileSize } from '../lib/format'
import VideoPlayer from './VideoPlayer'
import type { FileInfo } from '../api/files'
import type { TranscodeTask } from '../api/transcode'
import type { MediaInfo } from '../api/media'

interface PreviewModalProps {
  open: boolean
  file: FileInfo | null
  onClose: () => void
}

type TextState = { status: 'idle' } | { status: 'loading' } | { status: 'loaded'; content: string }

export default function PreviewModal({ open, file, onClose }: PreviewModalProps) {
  const [textState, setTextState] = useState<TextState>({ status: 'idle' })
  const [transcodeTask, setTranscodeTask] = useState<TranscodeTask | null>(null)
  const [mediaInfo, setMediaInfo] = useState<MediaInfo | null>(null)
  const [transcoding, setTranscoding] = useState(false)
  const [scanning, setScanning] = useState(false)

  const loadTranscodeStatus = useCallback(async (fileId: string) => {
    try {
      const tasks = await listTranscodes(fileId)
      const completed = tasks.find(t => t.status === 'completed')
      const processing = tasks.find(t => t.status === 'processing')
      const pending = tasks.find(t => t.status === 'pending')
      setTranscodeTask(completed || processing || pending || null)
    } catch {
      // ignore
    }
  }, [])

  useEffect(() => {
    if (!open || !file) return
    setTranscodeTask(null)
    setMediaInfo(null)

    const type = getPreviewType(file.content_type)
    let cancelled = false

    if (type === 'text') {
      const load = async () => {
        try {
          const token = localStorage.getItem('token')
          const res = await fetch(getPreviewUrl(file.id), {
            headers: { 'Authorization': `Bearer ${token}` },
          })
          const text = await res.text()
          if (!cancelled) setTextState({ status: 'loaded', content: text })
        } catch {
          if (!cancelled) setTextState({ status: 'loaded', content: '无法加载文件内容' })
        }
      }
      Promise.resolve().then(() => {
        if (!cancelled) setTextState({ status: 'loading' })
      })
      load()
    }

    // Load transcode info for video files
    if (type === 'video' || isTranscodableVideo(file.content_type)) {
      loadTranscodeStatus(file.id)
      getMediaInfo(file.id).then(info => {
        if (!cancelled) setMediaInfo(info)
      }).catch(() => {})
    }

    return () => { cancelled = true }
  }, [open, file, loadTranscodeStatus])

  // Poll transcode progress
  useEffect(() => {
    if (!transcodeTask || !file) return
    if (transcodeTask.status !== 'processing' && transcodeTask.status !== 'pending') return

    const interval = setInterval(() => {
      loadTranscodeStatus(file.id)
    }, 3000)

    return () => clearInterval(interval)
  }, [transcodeTask, file, loadTranscodeStatus])

  if (!open || !file) return null

  const type = getPreviewType(file.content_type)
  const transcodable = isTranscodableVideo(file.content_type)
  const previewUrl = getPreviewUrl(file.id)
  const token = localStorage.getItem('token')
  const authedUrl = `${previewUrl}?token=${encodeURIComponent(token || '')}`

  const handleStartTranscode = async (profile?: string) => {
    setTranscoding(true)
    try {
      const task = await createTranscode(file.id, profile)
      setTranscodeTask(task)
    } catch {
      // error handled silently
    } finally {
      setTranscoding(false)
    }
  }

  const handleScanMedia = async () => {
    setScanning(true)
    try {
      const info = await scanMedia(file.id)
      setMediaInfo(info)
    } catch {
      // error handled silently
    } finally {
      setScanning(false)
    }
  }

  const hlsUrl = transcodeTask?.status === 'completed' && transcodeTask.output_key
    ? `/api/v1/files/${file.id}/stream/index.m3u8`
    : null

  const subtitleUrl = transcodeTask?.status === 'completed'
    ? `/api/v1/files/${file.id}/stream/subtitles/default.vtt`
    : undefined

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60" onClick={onClose}>
      <div
        className="bg-white rounded-xl shadow-xl max-w-5xl w-full max-h-[90vh] flex flex-col mx-4"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-200">
          <div className="min-w-0 flex-1">
            <h3 className="font-semibold text-gray-900 truncate">{file.name}</h3>
            <p className="text-xs text-gray-500">{formatFileSize(file.size)} - {file.content_type}</p>
          </div>
          <button onClick={onClose} className="p-1 text-gray-400 hover:text-gray-600 rounded ml-2">
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-auto p-4">
          <div className="flex items-center justify-center min-h-[300px]">
            {/* HLS Player for transcoded videos */}
            {hlsUrl && (
              <VideoPlayer src={hlsUrl} subtitles={subtitleUrl} />
            )}

            {/* Native video player for browser-playable videos without transcode */}
            {!hlsUrl && type === 'video' && (
              <video src={authedUrl} controls className="max-w-full max-h-[70vh]" />
            )}

            {/* Non-video content */}
            {type === 'image' && (
              <img src={authedUrl} alt={file.name} className="max-w-full max-h-[70vh] object-contain" />
            )}
            {type === 'audio' && (
              <audio src={authedUrl} controls className="w-full max-w-md" />
            )}
            {type === 'pdf' && (
              <iframe src={authedUrl} className="w-full h-[70vh] border-0" title={file.name} />
            )}
            {type === 'text' && (
              textState.status === 'loading' ? (
                <p className="text-gray-400">加载中...</p>
              ) : textState.status === 'loaded' ? (
                <pre className="w-full text-sm text-gray-800 bg-gray-50 p-4 rounded-lg overflow-auto max-h-[70vh] whitespace-pre-wrap font-mono leading-relaxed">
                  {textState.content}
                </pre>
              ) : null
            )}
            {type === 'none' && !transcodable && (
              <p className="text-gray-400">此文件类型不支持预览</p>
            )}

            {/* Transcodable but not yet transcoded */}
            {!hlsUrl && transcodable && type === 'none' && (
              <div className="text-center text-gray-500">
                <Film className="w-12 h-12 mx-auto mb-3 text-gray-300" />
                <p className="mb-2">此视频格式需要转码后才能播放</p>
              </div>
            )}
          </div>

          {/* Transcode controls for video files */}
          {(type === 'video' || transcodable) && (
            <div className="mt-4 border-t border-gray-100 pt-4">
              <div className="flex items-center gap-3 flex-wrap">
                {/* Transcode status */}
                {transcodeTask && (
                  <TranscodeBadge task={transcodeTask} />
                )}

                {/* Start transcode button */}
                {(!transcodeTask || transcodeTask.status === 'failed') && (
                  <button
                    onClick={() => handleStartTranscode()}
                    disabled={transcoding}
                    className="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                  >
                    {transcoding ? (
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                    ) : (
                      <Play className="w-3.5 h-3.5" />
                    )}
                    开始转码
                  </button>
                )}

                {/* Scan media button */}
                <button
                  onClick={handleScanMedia}
                  disabled={scanning}
                  className="inline-flex items-center gap-1.5 px-3 py-1.5 text-sm bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 disabled:opacity-50"
                >
                  {scanning ? (
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  ) : (
                    <Film className="w-3.5 h-3.5" />
                  )}
                  识别媒体
                </button>
              </div>

              {/* Media info panel */}
              {mediaInfo && (mediaInfo.title || mediaInfo.overview) && (
                <div className="mt-3 p-3 bg-gray-50 rounded-lg flex gap-3">
                  {mediaInfo.poster_url && (
                    <img
                      src={mediaInfo.poster_url}
                      alt={mediaInfo.title || ''}
                      className="w-16 h-24 object-cover rounded shrink-0"
                    />
                  )}
                  <div className="min-w-0">
                    <p className="font-medium text-gray-900 text-sm">
                      {mediaInfo.title}
                      {mediaInfo.year && <span className="text-gray-500 ml-1">({mediaInfo.year})</span>}
                    </p>
                    {mediaInfo.season != null && mediaInfo.episode != null && (
                      <p className="text-xs text-gray-500">
                        S{String(mediaInfo.season).padStart(2, '0')}E{String(mediaInfo.episode).padStart(2, '0')}
                      </p>
                    )}
                    {mediaInfo.overview && (
                      <p className="text-xs text-gray-500 mt-1 line-clamp-3">{mediaInfo.overview}</p>
                    )}
                    <div className="flex gap-2 mt-1 text-xs text-gray-400">
                      {mediaInfo.resolution && <span>{mediaInfo.resolution}</span>}
                      {mediaInfo.duration && <span>{Math.floor(mediaInfo.duration / 60)}分钟</span>}
                    </div>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}

function TranscodeBadge({ task }: { task: TranscodeTask }) {
  if (task.status === 'completed') {
    return (
      <span className="inline-flex items-center gap-1 px-2 py-1 text-xs bg-green-100 text-green-700 rounded-full">
        已转码 ({task.profile})
      </span>
    )
  }
  if (task.status === 'processing') {
    return (
      <span className="inline-flex items-center gap-1 px-2 py-1 text-xs bg-blue-100 text-blue-700 rounded-full">
        <Loader2 className="w-3 h-3 animate-spin" />
        转码中 {task.progress}%
      </span>
    )
  }
  if (task.status === 'pending') {
    return (
      <span className="inline-flex items-center gap-1 px-2 py-1 text-xs bg-yellow-100 text-yellow-700 rounded-full">
        <Loader2 className="w-3 h-3 animate-spin" />
        等待中
      </span>
    )
  }
  if (task.status === 'failed') {
    return (
      <span className="inline-flex items-center gap-1 px-2 py-1 text-xs bg-red-100 text-red-700 rounded-full">
        <AlertCircle className="w-3 h-3" />
        转码失败
      </span>
    )
  }
  return null
}
