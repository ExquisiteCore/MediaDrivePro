import { useState } from 'react'

interface AvatarProps {
  userId: string
  username: string
  avatar: string | null
  size?: number
  className?: string
}

export default function Avatar({ userId, username, avatar, size = 32, className = '' }: AvatarProps) {
  const [failed, setFailed] = useState(false)

  const initial = username?.charAt(0).toUpperCase() || '?'
  const showImage = avatar && !failed

  const fontSize = size < 40 ? 'text-xs' : size < 56 ? 'text-lg' : 'text-2xl'

  return (
    <div
      className={`rounded-full bg-gradient-to-br from-[#b3d4fc] to-[#5b8db8] shrink-0 ${className}`}
      style={{ width: size, height: size, padding: size * 0.05 }}
    >
      <div className="w-full h-full rounded-full bg-white flex items-center justify-center overflow-hidden">
        {showImage ? (
          <img
            src={`/api/v1/users/${userId}/avatar?t=${encodeURIComponent(avatar)}`}
            alt=""
            className="w-full h-full object-cover"
            onError={() => setFailed(true)}
          />
        ) : (
          <span className={`font-bold text-[#5b8db8]/50 ${fontSize}`}>
            {initial}
          </span>
        )}
      </div>
    </div>
  )
}
