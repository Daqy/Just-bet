<script lang="ts" setup>
import { ref } from 'vue'
import { useAuthStore } from '~stores/useAuthStore'
import { useApi } from '@/services/api'
import { socket, state } from '@/socket'
import { useRouter } from 'vue-router'

const authStore = useAuthStore()
const router = useRouter()

const { post } = useApi('/api/battleships/create-game')
const betAmount = ref()

function preventAlphaKey(event: any) {
  const allowedValues = '1234567890'.split('')
  if (!allowedValues.includes(event.key) && event.key !== 'Backspace') {
    event.preventDefault()
  }
}

const error = ref<undefined | string>(undefined)

function createGame() {
  if (!betAmount.value) {
    error.value = 'must have a value'
    return
  }
  if (!authStore.balance) {
    error.value = 'you have no funds'
    return
  }

  if (betAmount.value <= 0) {
    error.value = 'enter a value over 0'
    return
  }

  if (betAmount.value > authStore.balance) {
    error.value = 'not enough cash'
    return
  }

  error.value = undefined

  post({ stake: betAmount.value }).then((response) => {
    const balanceUpdate = useApi('/api/get-balance')
    balanceUpdate.get().then((res: { balance: number }) => {
      authStore.balance = res.balance
      router.push(`/battleships/${response.gameid}`)
    })
  })
}
</script>

<template>
  <div class="controls">
    <input
      type="text"
      v-model.number="betAmount"
      @keydown="preventAlphaKey"
      placeholder="Bet amount.."
      class="bet-input"
      :class="{ inputError: error }"
    />
    <p v-if="error" class="error">{{ error }}</p>

    <button class="button" @click="createGame">create game</button>
  </div>
</template>

<style lang="scss" scoped>
.inputError {
  border: 2px solid var(--color-bomb-background) !important;
}

.error {
  /* color: var(--color-bomb-background); */
  color: var(--color-text-subtle);
  text-transform: capitalize;
}
.controls {
  display: flex;
  align-items: center;
  flex-direction: column;
  width: 100%;

  > *:not(p) {
    background: var(--color-container-subtle);
    color: white;
    overflow: hidden;
    width: 80%;
    box-shadow: rgba(0, 0, 0, 0.1) 0px 4px 12px;
    border-radius: 5px;
    min-height: 40px;
    border: none;
    aspect-ratio: 8/1;
    font-weight: bold;
    font-size: 1em;
    display: flex;
    justify-content: center;
    align-items: center;
  }
  > *:not(:first-child) {
    margin-top: 10px;
  }
}
.button {
  text-transform: capitalize;
}

.button:not(:disabled) {
  border: 2px solid var(--color-create-game-button);
  color: var(--color-create-game-button);

  &:hover {
    cursor: pointer;
    background: var(--color-create-game-button);
    color: var(--color-create-game-button-text);
  }
}

.button:disabled {
  color: var(--color-text-subtle);
}

.bet-input {
  padding: 0px 1rem;
}
</style>
