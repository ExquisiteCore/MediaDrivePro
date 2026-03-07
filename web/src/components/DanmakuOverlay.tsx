import { useEffect, useRef } from 'react'
import type { DanmakuItem } from '../hooks/useRoomSocket'

interface DanmakuOverlayProps {
  items: DanmakuItem[]
  onRemove: (id: string) => void
}

export default function DanmakuOverlay({ items, onRemove }: DanmakuOverlayProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const renderedRef = useRef(new Set<string>())

  useEffect(() => {
    const container = containerRef.current
    if (!container) return

    for (const item of items) {
      if (renderedRef.current.has(item.id)) continue
      renderedRef.current.add(item.id)

      const span = document.createElement('span')
      span.textContent = item.content
      span.style.position = 'absolute'
      span.style.whiteSpace = 'nowrap'
      span.style.color = item.color || '#ffffff'
      span.style.fontSize = '20px'
      span.style.fontWeight = 'bold'
      span.style.textShadow = '1px 1px 2px rgba(0,0,0,0.8)'
      span.style.pointerEvents = 'none'
      span.style.willChange = 'transform'

      // Random track (10 tracks)
      const track = Math.floor(Math.random() * 10)
      span.style.top = `${8 + track * 9}%`
      span.style.right = '0'

      span.style.animation = 'danmaku-scroll 8s linear forwards'
      container.appendChild(span)

      const handleEnd = () => {
        span.remove()
        renderedRef.current.delete(item.id)
        onRemove(item.id)
      }
      span.addEventListener('animationend', handleEnd)
    }
  }, [items, onRemove])

  return (
    <>
      <style>{`
        @keyframes danmaku-scroll {
          from { transform: translateX(100%); }
          to { transform: translateX(calc(-100vw - 100%)); }
        }
      `}</style>
      <div
        ref={containerRef}
        className="absolute inset-0 overflow-hidden pointer-events-none"
        style={{ zIndex: 10 }}
      />
    </>
  )
}
