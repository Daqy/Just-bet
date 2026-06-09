import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
// export const useSocket = () => {

// }
const baseURL = 'http://localhost:3000'

export const useSocketStore = defineStore('socket', () => {
  const socket = ref()

  const connected = computed(() => !!socket.value)

  const connect = (URL: string) => {
    socket.value = new WebSocket(baseURL + URL)

    socket.value.onopen = ({ target: io }: any) => {
      io.send(
        JSON.stringify({
          type: 'connected'
        })
      )
    }

    socket.value.onclose = (event) => {
      console.log('event close', event)
    }

    socket.value.onerror = (err) => {
      console.log('error', err)
    }
  }

  const emit = (type: string, data?: Record<string, unknown>) => {
    if (!connected.value) return
    console.log('socket', socket.value)
    socket.value.send(
      JSON.stringify({
        type,
        data
      })
    )
  }

  const on = (message: string, cb: (...args: any) => any) => {
    if (!connected.value) return
    socket.value.onmessage = (event: MessageEvent, ...args: any) => {
      if (event?.data?.type === message) {
        cb(event, ...args)
      }
    }
  }

  return {
    socket,
    connect,
    connected,
    emit,
    on
  }
})
// export const socket = new WebSocket(`${URL}/api/battleship/ws`)

// socket.onopen = () => {
//   console.log('WebSocket connection opened:', event)
//   state.connected = true
// }
