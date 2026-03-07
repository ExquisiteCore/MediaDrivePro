import { useState, useRef, useEffect } from 'react'
import { Send } from 'lucide-react'
import type { ChatMessage } from '../hooks/useRoomSocket'

interface ChatPanelProps {
  messages: ChatMessage[]
  onSendChat: (content: string) => void
  onSendDanmaku: (content: string, color?: string) => void
}

const DANMAKU_COLORS = [
  '#ffffff',
  '#ff0000',
  '#ff7f00',
  '#ffff00',
  '#00ff00',
  '#00bfff',
  '#8b00ff',
  '#ff69b4',
]

export default function ChatPanel({
  messages,
  onSendChat,
  onSendDanmaku,
}: ChatPanelProps) {
  const [input, setInput] = useState('')
  const [mode, setMode] = useState<'chat' | 'danmaku'>('chat')
  const [danmakuColor, setDanmakuColor] = useState('#ffffff')
  const listRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (listRef.current) {
      listRef.current.scrollTop = listRef.current.scrollHeight
    }
  }, [messages])

  const handleSend = () => {
    const text = input.trim()
    if (!text) return
    if (mode === 'chat') {
      onSendChat(text)
    } else {
      onSendDanmaku(text, danmakuColor)
    }
    setInput('')
  }

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault()
      handleSend()
    }
  }

  return (
    <div className="flex flex-col h-full">
      {/* Messages */}
      <div ref={listRef} className="flex-1 overflow-y-auto p-3 space-y-2 text-sm">
        {messages.map((msg, i) => (
          <div key={i}>
            {msg.type === 'system' ? (
              <div className="text-center text-xs text-gray-400 py-1">
                {msg.content}
              </div>
            ) : (
              <div>
                <span className="font-medium text-blue-600">
                  {msg.user?.name}
                </span>
                <span className="text-gray-400 mx-1">:</span>
                <span className="text-gray-700">{msg.content}</span>
              </div>
            )}
          </div>
        ))}
      </div>

      {/* Mode toggle + color picker */}
      <div className="px-3 pt-2 border-t border-gray-100 flex items-center gap-2">
        <button
          onClick={() => setMode(mode === 'chat' ? 'danmaku' : 'chat')}
          className={`text-xs px-2 py-1 rounded ${
            mode === 'danmaku'
              ? 'bg-orange-100 text-orange-600'
              : 'bg-gray-100 text-gray-600'
          }`}
        >
          {mode === 'chat' ? '聊天' : '弹幕'}
        </button>
        {mode === 'danmaku' && (
          <div className="flex gap-1">
            {DANMAKU_COLORS.map((c) => (
              <button
                key={c}
                onClick={() => setDanmakuColor(c)}
                className={`w-4 h-4 rounded-full border-2 ${
                  danmakuColor === c ? 'border-blue-500' : 'border-transparent'
                }`}
                style={{ backgroundColor: c }}
              />
            ))}
          </div>
        )}
      </div>

      {/* Input */}
      <div className="p-3 flex gap-2">
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder={mode === 'chat' ? '发送消息...' : '发送弹幕...'}
          className="flex-1 text-sm border border-gray-200 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500"
          maxLength={200}
        />
        <button
          onClick={handleSend}
          className="p-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
        >
          <Send className="w-4 h-4" />
        </button>
      </div>
    </div>
  )
}
