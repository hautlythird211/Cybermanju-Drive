import { ref } from 'vue'
import { useAppStore } from '@/stores/app'

const currentUser = ref<string | null>(null)
const isLoggedIn = ref(false)

export function useLogin() {
  const store = useAppStore()

  async function login(username: string) {
    localStorage.setItem('cybermanju_username', username)
    currentUser.value = username
    isLoggedIn.value = true

    try {
      const { invoke } = await import('@/composables/useTauri')
      const users = await invoke<any[]>('list_users')
      if (!users.find((u: any) => u.username === username)) {
        await invoke('register_user', {
          username,
          password: '',
          displayName: username,
          role: 'user',
        })
      }
    } catch {
      try {
        const data = await import('@/wasm/data')
        const users = await data.listUsers()
        if (!users.find((u: any) => u.username === username)) {
          await data.createUser({ username, password: '', displayName: username })
        }
      } catch {}
    }

    await store.fetchUsers()
  }

  function restoreSession() {
    const saved = localStorage.getItem('cybermanju_username')
    if (saved) {
      currentUser.value = saved
      isLoggedIn.value = true
    }
    return saved
  }

  return {
    currentUser,
    isLoggedIn,
    login,
    restoreSession,
  }
}
