import { ref } from 'vue'
import { defineStore } from 'pinia'
import { useApi } from '@/services/api'

export const useGameStore = defineStore('game', () => {
  const game = ref(undefined)
  const loading = ref(false)

  function getLatestGame() {
    const latestGame = useApi('/api/latest-game')
    loading.value = true

    latestGame.get().then((response: any) => {
      loading.value = false
      game.value = response
    })
  }

  function getGame(id: number) {
    const newGame = useApi(`/api/game/${id}`)
    loading.value = true

    newGame.get().then((response: any) => {
      loading.value = false
      game.value = response
    })
  }
  return { game, loading, getLatestGame, getGame }
})
