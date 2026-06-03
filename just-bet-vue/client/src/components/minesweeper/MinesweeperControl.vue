<script lang="ts" setup>
import { ref, watch, onMounted } from 'vue'
import { useGameStore } from '~stores/useGameStore'
import { useAuthStore } from '~stores/useAuthStore'
import { useApi } from '@/services/api'

const gameStore = useGameStore()
const authStore = useAuthStore()

const { post, data } = useApi('/api/create-game')
const betAmount = ref()

watch(data, (to, from) => {
  if (to.gameid) {
    gameStore.getGame(to.gameid)
  }
})

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

  post({
    bomb_count: bombMultipliers[activeMultiplier.value],
    stake: betAmount.value
  }).then((response: any) => {
    const balanceUpdate = useApi('/api/balance')
    balanceUpdate
      .post({ gameid: response.gameid, balance: -betAmount.value })
      .then((res: { balance: number }) => {
        authStore.balance = res.balance
      })
  })
}

const bombMultipliers = [1, 3, 5, 24]
const activeMultiplier = ref(0)

onMounted(() => {
  const getMultiplier = localStorage.getItem('multiplier')
  if (getMultiplier) {
    activeMultiplier.value = parseInt(getMultiplier)
  }
})

function changeMultiplier(index: number) {
  localStorage.setItem('multiplier', `${index}`)
  activeMultiplier.value = index
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

    <div class="number-container">
      <div
        v-for="(multiplier, index) in bombMultipliers"
        :key="index"
        @click="changeMultiplier(index)"
        :class="{ active: activeMultiplier === index }"
      >
        {{ multiplier }}
      </div>
    </div>
    <button class="button" :disabled="gameStore.game?.state === 'ongoing'" @click="createGame">
      create game
    </button>
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

.number-container {
  display: flex;

  > * {
    color: var(--color-text-subtle);
    min-height: 40px;
    flex-grow: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;

    &:hover {
      cursor: pointer;
      /* color: white; */
      background: var(--color-container-subtle-tail);
      /* box-shadow: rgb(0, 0, 0, 0.2) 3px 3px 6px 0px inset, */
      /* rgba(0, 0, 0, 0.2) -3px -3px 6px 1px inset; */
    }
  }
  > *:not(:last-child)::after {
    content: '';
    position: absolute;
    z-index: 10;
    background-color: var(--color-container-subtle-hightlight);
    width: 2px;
    height: 80%;
    top: 50%;
    transform: translate(0%, -50%);
    right: -1px;
  }
}

.active {
  box-shadow: rgb(0, 0, 0, 0.2) 3px 3px 6px 0px inset, rgba(0, 0, 0, 0.2) -3px -3px 6px 1px inset;
}
</style>
