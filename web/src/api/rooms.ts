import { request } from './client'

export interface Room {
  id: string
  host_id: string
  host_name: string
  name: string
  invite_code: string
  status: string
  current_file_id: string | null
  current_time: number
  max_members: number
  member_count: number
  created_at: string
}

export interface RoomMember {
  user_id: string
  username: string
  avatar: string | null
  role: string
}

export interface RoomDetail {
  room: Room
  members: RoomMember[]
}

export async function createRoom(name: string, maxMembers?: number): Promise<Room> {
  return request<Room>('/rooms', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ name, max_members: maxMembers }),
  })
}

export async function listRooms(): Promise<Room[]> {
  return request<Room[]>('/rooms')
}

export async function getRoom(id: string): Promise<RoomDetail> {
  return request<RoomDetail>(`/rooms/${id}`)
}

export async function closeRoom(id: string): Promise<void> {
  return request<void>(`/rooms/${id}`, { method: 'DELETE' })
}

export async function joinRoom(inviteCode: string): Promise<Room> {
  return request<Room>('/rooms/join', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ invite_code: inviteCode }),
  })
}

export async function setPlaying(roomId: string, fileId: string): Promise<void> {
  return request<void>(`/rooms/${roomId}/play`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ file_id: fileId }),
  })
}

export async function listMembers(roomId: string): Promise<RoomMember[]> {
  return request<RoomMember[]>(`/rooms/${roomId}/members`)
}
