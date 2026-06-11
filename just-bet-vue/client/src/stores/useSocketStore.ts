import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
// export const useSocket = () => {

// }
const baseURL = 'http://localhost:3000'

export const useSocketStore = defineStore('socket', () => {
  const socket = ref()

  const connected = computed(() => socket.value?.readyState === 1)

  const connect = (URL: string, roomid = 'global') => {
    socket.value = new WebSocket(baseURL + URL)

    socket.value.onopen = ({ target: io }: any) => {
      io.send(
        JSON.stringify({
          type: 'connected',
          roomid
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
    console.log(socket.value)
    if (!connected.value) return false
    socket.value.send(
      JSON.stringify({
        type,
        data
      })
    )
    return true
  }

  const on = (message: string, cb: (...args: any) => any) => {
    if (!connected.value) return
    socket.value.onmessage = (event: MessageEvent, ...args: any) => {
      let data
      try {
        data = JSON.parse(event.data)
      } catch (err) {
        data = event.data
      }
      if (data?.type === message) {
        cb(data, event, ...args)
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
