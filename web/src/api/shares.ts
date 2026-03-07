import { request } from './client'

export interface ShareInfo {
  id: string
  file_id: string | null
  folder_id: string | null
  token: string
  has_password: boolean
  permission: string
  max_downloads: number | null
  download_count: number
  expires_at: string | null
  created_at: string
}

export interface PublicShareInfo {
  token: string
  has_password: boolean
  file_name: string | null
  file_size: number | null
  expires_at: string | null
}

export interface CreateShareRequest {
  file_id?: string
  folder_id?: string
  password?: string
  max_downloads?: number
  expires_at?: string
}

export async function createShare(data: CreateShareRequest): Promise<ShareInfo> {
  return request<ShareInfo>('/shares', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
}

export async function listShares(): Promise<ShareInfo[]> {
  return request<ShareInfo[]>('/shares')
}

export async function deleteShare(id: string): Promise<void> {
  return request<void>(`/shares/${id}`, { method: 'DELETE' })
}

// Public share endpoints (no auth)
export async function getPublicShare(token: string): Promise<PublicShareInfo> {
  const res = await fetch(`/api/v1/shares/public/${token}`)
  if (!res.ok) {
    const body = await res.json()
    throw new Error(body.error?.message || '分享不存在')
  }
  const json = await res.json()
  return json.data
}

export async function verifySharePassword(token: string, password: string): Promise<PublicShareInfo> {
  const res = await fetch(`/api/v1/shares/public/${token}/verify`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ password }),
  })
  if (!res.ok) {
    const body = await res.json()
    throw new Error(body.error?.message || '密码错误')
  }
  const json = await res.json()
  return json.data
}

export function getShareDownloadUrl(token: string): string {
  return `/api/v1/shares/public/${token}/download`
}
