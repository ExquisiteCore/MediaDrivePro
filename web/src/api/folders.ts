import { request } from './client'
import type { FileInfo } from './files'

export interface FolderInfo {
  id: string
  name: string
  parent_id: string | null
  created_at: string
  updated_at: string
}

export interface FolderChildren {
  folders: FolderInfo[]
  files: FileInfo[]
}

export async function createFolder(data: { name: string; parent_id?: string }): Promise<FolderInfo> {
  return request<FolderInfo>('/folders', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
}

export async function listRootChildren(): Promise<FolderChildren> {
  return request<FolderChildren>('/folders')
}

export async function listRootFolders(): Promise<FolderInfo[]> {
  const children = await listRootChildren()
  return children.folders
}

export async function getFolder(id: string): Promise<FolderInfo> {
  return request<FolderInfo>(`/folders/${id}`)
}

export async function updateFolder(id: string, data: { name?: string; parent_id?: string | null }): Promise<FolderInfo> {
  return request<FolderInfo>(`/folders/${id}`, {
    method: 'PUT',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(data),
  })
}

export async function listFolderChildren(id: string): Promise<FolderChildren> {
  return request<FolderChildren>(`/folders/${id}/children`)
}

export async function deleteFolder(id: string): Promise<void> {
  return request<void>(`/folders/${id}`, { method: 'DELETE' })
}
