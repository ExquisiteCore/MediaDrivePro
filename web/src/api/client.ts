const BASE = '/api/v1'

export interface ApiError {
  code: string
  message: string
}

export class ApiRequestError extends Error {
  code: string
  constructor(err: ApiError) {
    super(err.message)
    this.code = err.code
  }
}

export async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const token = localStorage.getItem('token')
  const headers: Record<string, string> = {}
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }
  if (options?.headers) {
    Object.assign(headers, options.headers)
  }

  const res = await fetch(BASE + path, { ...options, headers })

  if (res.status === 401) {
    localStorage.removeItem('token')
    window.location.href = '/login'
    throw new ApiRequestError({ code: 'unauthorized', message: '登录已过期' })
  }

  if (res.status === 204) {
    return undefined as T
  }

  if (!res.ok) {
    const body = await res.json()
    throw new ApiRequestError(body.error)
  }

  const json = await res.json()
  return json.data
}

export interface PaginatedResponse<T> {
  data: T[]
  meta: { page: number; per_page: number; total: number }
}

export async function requestPaginated<T>(path: string, options?: RequestInit): Promise<PaginatedResponse<T>> {
  const token = localStorage.getItem('token')
  const headers: Record<string, string> = {}
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }
  if (options?.headers) {
    Object.assign(headers, options.headers)
  }

  const res = await fetch(BASE + path, { ...options, headers })

  if (res.status === 401) {
    localStorage.removeItem('token')
    window.location.href = '/login'
    throw new ApiRequestError({ code: 'unauthorized', message: '登录已过期' })
  }

  if (!res.ok) {
    const body = await res.json()
    throw new ApiRequestError(body.error)
  }

  return res.json()
}
