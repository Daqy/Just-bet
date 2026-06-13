import { reactive, watch } from 'vue'
import { io } from 'socket.io-client'
import { useAuthStore } from './stores/useAuthStore'
import { useApi } from '@/services/api'
import { useSocket } from './composable/useSocket'

// socket.on('connect', () => {
//   state.connected = true
// })

// socket.on('disconnect', () => {
//   state.connected = false
// })

// socket.on('user-joined', async ({ game }) => {
//   const { get } = useApi(`/api/game/${game.id}`)

//   get().then((response) => {
//     state.game = response
//   })
// })

// socket.on('user-ready', async (gameid) => {
//   console.log('ready')
//   const { get } = useApi(`/api/game/${gameid}`)

//   get().then((response) => {
//     state.game = response
//   })
// })

// socket.on('user-has-won', async () => {
//   const authStore = useAuthStore()
//   const { get } = useApi('/api/get-balance')
//   get().then((res: { balance: number }) => {
//     authStore.balance = res.balance
//   })
// })

// socket.on('board-click', async ({ game }) => {
//   const { get } = useApi(`/api/game/${game.id}`)

//   get().then((response) => {
//     state.game = response
//   })
// })
