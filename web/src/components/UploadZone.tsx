import { useCallback, useState, useRef } from 'react'
import { Upload, X } from 'lucide-react'
import { uploadFile, initMultipartUpload, uploadPart, completeMultipartUpload } from '../api/files'
import type { FileInfo } from '../api/files'

const MULTIPART_THRESHOLD = 10 * 1024 * 1024 // 10MB
const CHUNK_SIZE = 5 * 1024 * 1024 // 5MB
const CONCURRENT_UPLOADS = 3

interface UploadZoneProps {
  folderId?: string
  onUploadComplete: (file: FileInfo) => void
}

interface UploadTask {
  file: File
  progress: number
  status: 'pending' | 'uploading' | 'done' | 'error'
  error?: string
}

export default function UploadZone({ folderId, onUploadComplete }: UploadZoneProps) {
  const [tasks, setTasks] = useState<UploadTask[]>([])
  const [dragOver, setDragOver] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  const updateTask = useCallback((index: number, update: Partial<UploadTask>) => {
    setTasks(prev => prev.map((t, i) => i === index ? { ...t, ...update } : t))
  }, [])

  const doUpload = useCallback(async (file: File, index: number) => {
    updateTask(index, { status: 'uploading', progress: 0 })

    try {
      let result: FileInfo

      if (file.size > MULTIPART_THRESHOLD) {
        // Multipart upload
        const { upload_id } = await initMultipartUpload({
          file_name: file.name,
          folder_id: folderId,
          content_type: file.type || 'application/octet-stream',
        })

        const totalParts = Math.ceil(file.size / CHUNK_SIZE)
        let completed = 0

        // Upload in batches of CONCURRENT_UPLOADS
        for (let i = 0; i < totalParts; i += CONCURRENT_UPLOADS) {
          const batch = []
          for (let j = i; j < Math.min(i + CONCURRENT_UPLOADS, totalParts); j++) {
            const start = j * CHUNK_SIZE
            const end = Math.min(start + CHUNK_SIZE, file.size)
            const chunk = await file.slice(start, end).arrayBuffer()
            batch.push(uploadPart(upload_id, j + 1, chunk).then(() => {
              completed++
              updateTask(index, { progress: Math.round((completed / totalParts) * 100) })
            }))
          }
          await Promise.all(batch)
        }

        result = await completeMultipartUpload(upload_id)
      } else {
        // Simple upload
        result = await uploadFile(file, folderId)
        updateTask(index, { progress: 100 })
      }

      updateTask(index, { status: 'done', progress: 100 })
      onUploadComplete(result)
    } catch (err: unknown) {
      updateTask(index, { status: 'error', error: err instanceof Error ? err.message : '上传失败' })
    }
  }, [folderId, onUploadComplete, updateTask])

  const handleFiles = useCallback((files: FileList | File[]) => {
    const fileArr = Array.from(files)
    const startIndex = tasks.length

    setTasks(prev => [
      ...prev,
      ...fileArr.map(f => ({ file: f, progress: 0, status: 'pending' as const })),
    ])

    fileArr.forEach((file, i) => {
      doUpload(file, startIndex + i)
    })
  }, [tasks.length, doUpload])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    setDragOver(false)
    if (e.dataTransfer.files.length > 0) {
      handleFiles(e.dataTransfer.files)
    }
  }, [handleFiles])

  const clearDone = () => {
    setTasks(prev => prev.filter(t => t.status !== 'done' && t.status !== 'error'))
  }

  const activeTasks = tasks.filter(t => t.status === 'uploading' || t.status === 'pending')
  const hasFinished = tasks.some(t => t.status === 'done' || t.status === 'error')

  return (
    <div>
      {/* Drop zone */}
      <div
        onDragOver={(e) => { e.preventDefault(); setDragOver(true) }}
        onDragLeave={() => setDragOver(false)}
        onDrop={handleDrop}
        onClick={() => inputRef.current?.click()}
        className={`border-2 border-dashed rounded-lg p-6 text-center cursor-pointer transition-colors ${
          dragOver ? 'border-blue-400 bg-blue-50' : 'border-gray-300 hover:border-gray-400'
        }`}
      >
        <Upload className="w-8 h-8 mx-auto text-gray-400 mb-2" />
        <p className="text-sm text-gray-500">拖拽文件到此处或点击上传</p>
        <p className="text-xs text-gray-400 mt-1">大于 10MB 的文件自动使用分片上传</p>
      </div>
      <input
        ref={inputRef}
        type="file"
        multiple
        className="hidden"
        onChange={(e) => e.target.files && handleFiles(e.target.files)}
      />

      {/* Upload progress */}
      {tasks.length > 0 && (
        <div className="mt-3 space-y-2">
          {hasFinished && activeTasks.length === 0 && (
            <button onClick={clearDone} className="text-xs text-gray-500 hover:text-gray-700">
              清除已完成
            </button>
          )}
          {tasks.map((task, i) => (
            <div key={i} className="flex items-center gap-2 text-sm">
              <span className="truncate flex-1 text-gray-700">{task.file.name}</span>
              {task.status === 'uploading' && (
                <div className="w-24 bg-gray-200 rounded-full h-1.5">
                  <div className="bg-blue-500 h-1.5 rounded-full transition-all" style={{ width: `${task.progress}%` }} />
                </div>
              )}
              {task.status === 'done' && <span className="text-green-600 text-xs">完成</span>}
              {task.status === 'error' && (
                <span className="text-red-600 text-xs flex items-center gap-1">
                  <X className="w-3 h-3" />{task.error}
                </span>
              )}
              {task.status === 'pending' && <span className="text-gray-400 text-xs">等待中</span>}
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
