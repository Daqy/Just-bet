import { ref, computed } from 'vue'
import { defineStore } from 'pinia'
import { useApi } from '@/services/api'

const defaultValues = {
  token: null,
  username: '',
  balance: null
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(defaultValues.token)
  const username = ref(defaultValues.username)
  const balance = ref<number | null>(defaultValues.balance)

  function _reset() {
    token.value = defaultValues.token
    username.value = defaultValues.username
    balance.value = defaultValues.balance
  }

  function login(username: string, password: string) {
    const { post } = useApi('/api/login')

    return new Promise((resolve, reject) => {
      post({ username, password })
        .then((response: { token: string }) => {
          token.value = 'auth'
          resolve(true)
        })
        .catch((error) => {
          resolve(false)
        })
    })
  }

  function isAuthenticated() {
    const { post } = useApi('/api/auth')

    return new Promise((resolve, reject) =>
      post({ token: token.value })
        .then((response: { token: string }) => {
          token.value = 'auth'
          resolve(true)
        })
        .catch((error) => {
          resolve(false)
        })
    )
  }

  return { token, username, balance, _reset, isAuthenticated, login }
})
