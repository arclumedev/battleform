const API_BASE = import.meta.env.VITE_API_URL || '/api'
class ApiClient {
  private base: string
  constructor(base: string) { this.base = base }
  async get(path: string) {
    const res = await fetch(`${this.base}${path}`, { credentials: 'include' })
    if (!res.ok) throw new Error(`API error: ${res.status}`)
    return { data: await res.json() }
  }
  async post(path: string, body?: Record<string, unknown>) {
    const res = await fetch(`${this.base}${path}`, {
      method: 'POST', credentials: 'include',
      headers: body ? { 'Content-Type': 'application/json' } : {},
      body: body ? JSON.stringify(body) : undefined,
    })
    if (!res.ok) throw new Error(`API error: ${res.status}`)
    return { data: await res.json() }
  }
}
export const api = new ApiClient(API_BASE)
