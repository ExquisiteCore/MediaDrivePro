import { useEffect, useRef } from 'react'
import Hls from 'hls.js'

interface VideoPlayerProps {
  src: string
  subtitles?: string
}

export default function VideoPlayer({ src, subtitles }: VideoPlayerProps) {
  const videoRef = useRef<HTMLVideoElement>(null)
  const hlsRef = useRef<Hls | null>(null)

  useEffect(() => {
    const video = videoRef.current
    if (!video) return

    if (Hls.isSupported()) {
      const hls = new Hls({
        xhrSetup: (xhr) => {
          const token = localStorage.getItem('token')
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
      // Safari native HLS support
      const token = localStorage.getItem('token')
      video.src = `${src}${src.includes('?') ? '&' : '?'}token=${encodeURIComponent(token || '')}`
    }
  }, [src])

  return (
    <div className="w-full">
      <video
        ref={videoRef}
        controls
        className="max-w-full max-h-[70vh] mx-auto"
        crossOrigin="anonymous"
      >
        {subtitles && (
          <track
            kind="subtitles"
            src={`${subtitles}${subtitles.includes('?') ? '&' : '?'}token=${encodeURIComponent(localStorage.getItem('token') || '')}`}
            srcLang="zh"
            label="字幕"
            default
          />
        )}
      </video>
    </div>
  )
}
