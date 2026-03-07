import { request, requestPaginated } from './client'

export interface FileInfo {
  id: string
  name: string
  size: number
  content_type: string
  folder_id: string | null
  status: string
  transcode_status: string | null
  has_media_info: boolean
  created_at: string
  updated_at: string
}

export interface FileListQuery {
  folder_id?: string
  page?: number
  per_page?: number
  search?: string
  sort?: string
  order?: string
}

export async function listFiles(query: FileListQuery) {
  const params = new URLSearchParams()
  if (query.folder_id) params.set('folder_id', query.folder_id)
  if (query.page) params.set('page', String(query.page))
  if (query.per_page) params.set('per_page', String(query.per_page))
  if (query.search) params.set('search', query.search)
  if (query.sort) params.set('sort', query.sort)
  if (query.order) params.set('order', query.order)
  const qs = params.toString()
  return requestPaginated<FileInfo>(`/files${qs ? '?' + qs : ''}`)
}

export async function getFile(id: string): Promise<FileInfo> {
  return request<FileInfo>(`/files/${id}`)
}

export async function uploadFile(file: File, folderId?: string): Promise<FileInfo> {
  const formData = new FormData()
  formData.append('file', file)
  if (folderId) {
    formData.append('folder_id', folderId)
  }
  return request<FileInfo>('/files', {
    method: 'POST',
    body: formData,
  })
}

export async function updateFile(id: string, data: { name?: string; folder_id?: string | null }): Promise<FileInfo> {
  return request<FileInfo>(`/files/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
}

export async function deleteFile(id: string): Promise<void> {
  return request<void>(`/files/${id}`, { method: 'DELETE' })
}

export function getDownloadUrl(id: string): string {
  return `/api/v1/files/${id}/download`
}

export function getPreviewUrl(id: string): string {
  return `/api/v1/files/${id}/preview`
}

// Multipart upload
export interface InitMultipartResponse {
  upload_id: string
}

export async function initMultipartUpload(data: {
  file_name: string
  folder_id?: string
  content_type?: string
}): Promise<InitMultipartResponse> {
  return request<InitMultipartResponse>('/files/multipart/init', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
}

export async function uploadPart(uploadId: string, partNumber: number, data: ArrayBuffer): Promise<void> {
  const token = localStorage.getItem('token')
  const res = await fetch(`/api/v1/files/multipart/${uploadId}/${partNumber}`, {
    method: 'PUT',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/octet-stream',
    },
    body: data,
  })
  if (!res.ok) {
    const body = await res.json()
    throw new Error(body.error?.message || 'Upload part failed')
  }
}

export async function completeMultipartUpload(uploadId: string): Promise<FileInfo> {
  return request<FileInfo>(`/files/multipart/${uploadId}/complete`, {
    method: 'POST',
  })
}

export async function cancelMultipartUpload(uploadId: string): Promise<void> {
  return request<void>(`/files/multipart/${uploadId}`, {
    method: 'DELETE',
  })
}
