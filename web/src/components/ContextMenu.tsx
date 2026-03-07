import { useEffect, useRef } from 'react'

interface MenuItem {
  label: string
  icon: React.ReactNode
  onClick: () => void
  destructive?: boolean
}

interface ContextMenuProps {
  x: number
  y: number
  items: MenuItem[]
  onClose: () => void
}

export default function ContextMenu({ x, y, items, onClose }: ContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        onClose()
      }
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [onClose])

  // Adjust position so menu doesn't go off-screen
  const style: React.CSSProperties = {
    position: 'fixed',
    left: x,
    top: y,
    zIndex: 100,
  }

  return (
    <div ref={ref} style={style} className="bg-white rounded-lg shadow-lg border border-gray-200 py-1 min-w-[160px]">
      {items.map((item, i) => (
        <button
          key={i}
          onClick={() => { item.onClick(); onClose() }}
          className={`w-full flex items-center gap-2 px-3 py-2 text-sm transition-colors ${
            item.destructive
              ? 'text-red-600 hover:bg-red-50'
              : 'text-gray-700 hover:bg-gray-100'
          }`}
        >
          {item.icon}
          {item.label}
        </button>
      ))}
    </div>
  )
}
