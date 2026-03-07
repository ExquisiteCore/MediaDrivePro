import { formatFileSize } from '../lib/format'

interface StorageBarProps {
  used: number
  quota: number
}

export default function StorageBar({ used, quota }: StorageBarProps) {
  const percent = quota > 0 ? Math.min((used / quota) * 100, 100) : 0
  const color = percent > 90 ? 'bg-red-500' : percent > 70 ? 'bg-yellow-500' : 'bg-blue-500'

  return (
    <div>
      <div className="flex justify-between text-xs text-gray-500 mb-1">
        <span>存储空间</span>
        <span>{formatFileSize(used)} / {formatFileSize(quota)}</span>
      </div>
      <div className="w-full bg-gray-200 rounded-full h-1.5">
        <div
          className={`h-1.5 rounded-full transition-all ${color}`}
          style={{ width: `${percent}%` }}
        />
      </div>
    </div>
  )
}
