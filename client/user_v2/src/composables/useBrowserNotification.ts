import { ref, onMounted } from 'vue'

const granted = ref(false)

export function useBrowserNotification() {
  onMounted(() => {
    if (!('Notification' in window)) return
    if (Notification.permission === 'granted') {
      granted.value = true
    } else if (Notification.permission !== 'denied') {
      Notification.requestPermission().then((p) => {
        granted.value = p === 'granted'
      })
    }
  })

  function notify(title: string, options?: NotificationOptions) {
    if (!granted.value && Notification.permission === 'granted') {
      granted.value = true
    }
    if (!granted.value) return
    try {
      const n = new Notification(title, {
        icon: '/favicon.ico',
        ...options,
      })
      setTimeout(() => n.close(), 5000)
    } catch {
      // silently fail
    }
  }

  return { granted, notify }
}
