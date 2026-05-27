class StorageService {
  get<T = string>(key: string): T | null {
    try {
      const raw = localStorage.getItem(key)
      if (raw === null) return null
      return JSON.parse(raw) as T
    } catch {
      return localStorage.getItem(key) as T | null
    }
  }

  set(key: string, value: unknown): void {
    try {
      localStorage.setItem(key, JSON.stringify(value))
    } catch (e) {
      console.error('Storage set error:', e)
    }
  }

  remove(key: string): void {
    localStorage.removeItem(key)
  }

  clear(): void {
    localStorage.clear()
  }

  session = {
    get<T = string>(key: string): T | null {
      try {
        const raw = sessionStorage.getItem(key)
        if (raw === null) return null
        return JSON.parse(raw) as T
      } catch {
        return sessionStorage.getItem(key) as T | null
      }
    },
    set(key: string, value: unknown): void {
      try {
        sessionStorage.setItem(key, JSON.stringify(value))
      } catch (e) {
        console.error('Session storage set error:', e)
      }
    },
    remove(key: string): void {
      sessionStorage.removeItem(key)
    },
    clear(): void {
      sessionStorage.clear()
    },
  }
}

export const storageService = new StorageService()
