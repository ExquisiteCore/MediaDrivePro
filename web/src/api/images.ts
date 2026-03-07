import { request, requestPaginated, PaginatedResponse } from './client'

export interface ImageInfo {
  id: string
  hash: string
  original_name: string
  url: string
  thumb_url: string
  markdown: string
  size: number
  original_size: number
  width: number
  height: number
  created_at: string
}

export async function uploadImage(file: File): Promise<ImageInfo> {
  const form = new FormData()
  form.append('image', file)
  return request<ImageInfo>('/images', {
    method: 'POST',
    body: form,
  })
}

export async function listImages(page = 1, perPage = 20): Promise<PaginatedResponse<ImageInfo>> {
  return requestPaginated<ImageInfo>(`/images?page=${page}&per_page=${perPage}`)
}

export async function deleteImage(id: string): Promise<void> {
  return request<void>(`/images/${id}`, { method: 'DELETE' })
}
