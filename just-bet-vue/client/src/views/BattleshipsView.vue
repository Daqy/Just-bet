<script lang="ts" setup>
import { useApi } from '@/services/api'
import BattleshipControl from '@/components/battleships/battleshipsMainControls.vue'
import { state } from '@/socket'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/useAuthStore'
import { useSocketStore } from '@/stores/useSocketStore'

const authStore = useAuthStore()
const router = useRouter()
const latestGame = useApi('/api/latest-game?gameType=battleships')

const socket = useSocketStore()
socket.emit('get-games')

latestGame
  .get()
  .then((response) => {
    if (response.state !== 'done') {
      router.push(`/battleships/${response.id}`)
    }
  })
  .catch((error) => {})

const joinGame = (gameid) => {
  const { post } = useApi(`/api/battleships/join`)

  post({ gameid }).then((response) => {
    const balanceUpdate = useApi('/api/get-balance')
    balanceUpdate.get().then((res: { balance: number }) => {
      authStore.balance = res.balance
    })
    router.push(`/battleships/${response.gameid}`)
  })
}
</script>

<template>
  <main class="game">
    <section class="player-games">
      <div class="container">
        <template v-if="state.games.length > 0">
          <div class="player-game" v-for="game in state.games" :key="game">
            <div class="icon"></div>
            <div>
              <h2 class="heading">Created by</h2>
              <p>{{ game.belongsTo }}</p>
            </div>
            <div>
              <h2 class="heading">Bet amount</h2>
              <p>${{ game.stake }}</p>
            </div>
            <button class="button" @click="joinGame(game.id)">join</button>
          </div>
        </template>
        <div class="no-games" v-else>
          <p>No games found</p>
        </div>
      </div>
    </section>
    <section class="create-game">
      <div class="game-space">
        <BattleshipControl />
      </div>
    </section>
  </main>
</template>

<style lang="scss" scoped>
main {
  flex-grow: 1;
  display: flex;
}

.game {
  display: flex;
}

.player-game {
  background: var(--color-container-subtle);
  box-shadow: rgba(0, 0, 0, 0.1) 0px 4px 12px;
  padding: 1rem;
  border-radius: 5px;
  display: flex;
  align-items: center;
  gap: 1rem;
  width: 100%;
  color: var(--color-text-subtle);

  &:hover {
    transform: scale(101.5%);
    cursor: pointer;
  }

  .heading {
    color: white;
    font-size: 1rem;
  }

  .icon {
    width: 50px;
    background: var(--color-container-titles);
    aspect-ratio: 1/1;
  }
}

.container {
  width: 100%;
  height: 100%;
  overflow-y: scroll;
  overflow-x: hidden;
  display: flex;
  flex-direction: column;
  gap: 1rem;
  padding: 5px;
}

.game-space {
  display: flex;
  width: 80%;
  height: 80%;
  justify-content: center;
}

.button {
  background: var(--color-container-subtle);
  /* box-shadow: rgba(0, 0, 0, 0.1) 0px 4px 12px; */
  border-radius: 5px;
  font-weight: bold;
  border: 2px solid var(--color-create-game-button);
  color: var(--color-create-game-button);
  padding: 0.5rem 1rem;
  font-size: 1rem;
  margin-left: auto;

  &:hover {
    background: var(--color-create-game-button);
    color: var(--color-create-game-button-text);
    cursor: pointer;
  }
}

.player-games {
  height: 100%;
  aspect-ratio: 1/1;
  padding: 26px 1rem;
  display: flex;

  &::after {
    content: '';
    position: absolute;
    right: 0px;
    width: 2px;
    height: 90%;
    background: var(--color-container-titles);
  }

  .no-games {
    display: flex;
    justify-content: center;
    align-items: center;
    width: 100%;
    height: 100%;
  }
}

.create-game {
  height: 100%;
  flex-grow: 1;
  display: flex;
  justify-content: center;
  align-items: center;
}
</style>
