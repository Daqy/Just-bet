import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'

export const useSocket = (URL: string, room: String) => {
  const socket = ref(new WebSocket(location.origin + URL))
  const onMessageHandlers = ref<Record<string, (...args: any) => any>>({})

  const connected = ref(false)

  socket.value.onopen = ({ target: io }: any) => {
    io.send(
      JSON.stringify({
        type: 'connected',
        roomid: room
      })
    )

    connected.value = true
  }

  socket.value.onclose = (event) => {
    connected.value = false
    console.log('disconnected')
  }

  socket.value.onerror = (event) => {
    console.log('error')
  }

  const emit = (type: string, data?: Record<string, unknown>, attempt = 10) => {
    console.log(connected.value)
    let counter = 0
    ;(function loop() {
      setTimeout(() => {
        if (connected.value) {
          socket.value.send(
            JSON.stringify({
              type,
              data
            })
          )
        } else if (counter <= attempt) {
          loop()
          counter++
        }
      }, 1000)
    })()
  }

  socket.value.onmessage = (event: MessageEvent, ...args: any) => {
    let data
    try {
      data = JSON.parse(event.data)
    } catch (err) {
      data = event.data
    }
    if (!onMessageHandlers.value[data.type]) return

    onMessageHandlers.value[data.type](data, event, ...args)
  }

  const on = (message: string, cb: (...args: any) => any) => {
    onMessageHandlers.value[message] = cb
  }

  return {
    $socket: socket,
    connected,
    emit,
    on
  }
}
