<script lang="ts" setup>
import MineGameBoard from '~components/minesweeper/MineGameBoard.vue'
import MineBoardSquare from '~components/minesweeper/MineBoardSquare.vue'
import MinesweeperControl from '~components/minesweeper/MinesweeperControl.vue'
import MinesweeperInfomatics from '~components/minesweeper/MinesweeperInfomatics.vue'
import AppSkeletonLoader from '~components/AppSkeletonLoader.vue'
import AppCopyGameOutput from '~components/AppCopyGameOutput.vue'
import { prettify } from '~services/prettify'
import { useAuthStore } from '~stores/useAuthStore'
import { useGameStore } from '~stores/useGameStore'
import { onMounted } from 'vue'
import { useApi } from '~services/api'
import { ref } from 'vue'
import { hasBeenClicked } from '~services/hasBeenClicked'
import { getClickEarn } from '~services/getClickEarn'

const authStore = useAuthStore()
const gameStore = useGameStore()

const loadingClick = ref(false)

onMounted(() => {
  if (!gameStore.game) {
    gameStore.getLatestGame()
  }
})

function calculateDelay(totalNumber: number, numberOfMine: number = totalNumber) {
  return Math.pow((1 / totalNumber) * numberOfMine, 0.5) * 100
}

function wait(milliseconds: number) {
  return new Promise((resolve) => {
    setTimeout(resolve, milliseconds)
  })
}

const queue: number[] = []

function squareClick(id: number, addToQueue = true) {
  if (addToQueue) {
    queue.push(id)
  }
  if (
    hasBeenClicked(id, gameStore.game.clicks) ||
    gameStore.game.state === 'done' ||
    loadingClick.value
  )
    return

  const { get } = useApi(`/api/game/minesweeper/${gameStore.game.id}/click?click_position=${id}`)
  console.log(queue, id)
  if (queue.length > 1 && queue[0] !== id) {
    return
  }
  get().then(
    (response: {
      state: string
      pool: number
      clicks: { position: string; earned: number }[]
      nextClick: number
      result?: string
    }) => {
      if (response) {
        if (response.state === 'done') {
          gameStore.game = { ...gameStore.game, ...response }

          const { get } = useApi(`/api/game/${gameStore.game.id}/bomb-locations`)
          get().then(async (res: number[]) => {
            gameStore.game.bomb.position = []

            if (response.result === 'lost') {
              gameStore.game.bomb.position.push(id)
              for (let index = res.length - 1; index >= 0; index--) {
                if (id !== res[index]) {
                  await wait(calculateDelay(res.length, index))
                  gameStore.game.bomb.position.push(res[index])
                }
              }
            } else {
              gameStore.game.bomb.position = res
            }
          })
        } else {
          gameStore.game = { ...gameStore.game, ...response }
        }
      }
      queue.splice(0, 1)
      loadingClick.value = false
      console.log(queue, id)
      if (queue.length > 0) {
        console.log(queue, id)
        squareClick(queue[0], false)
      }
    }
  )
}

function claimRewards() {
  const { post } = useApi('/api/claim-game')
  post({ gameid: gameStore.game.id }).then((response: any) => {
    gameStore.game = { ...gameStore.game, ...response }
    const { get } = useApi(`/api/game/${response.id}/bomb-locations`)
    get().then((res: any) => {
      gameStore.game.bomb.position = res
    })
    const balanceUpdate = useApi('/api/balance')
    balanceUpdate
      .post({ gameid: response.id, balance: response.pool })
      .then((res: { balance: number }) => {
        authStore.balance = res.balance
      })
  })
}
</script>

<template>
  <main>
    <section class="game-container">
      <div class="big-container">
        <MineGameBoard :loading="gameStore.loading">
          <MineBoardSquare
            v-for="index in gameStore.game?.size"
            :key="index"
            :id="index"
            :isBomb="gameStore.game?.bomb?.position?.includes(index)"
            :flip="hasBeenClicked(index, gameStore.game.clicks)"
            :earn="getClickEarn(index, gameStore.game)"
            @square-click="squareClick"
          />
        </MineGameBoard>
      </div>
    </section>
    <section class="control-container">
      <div class="control-space">
        <MinesweeperControl />
        <div class="infomatics-container">
          <MinesweeperInfomatics />
          <div class="claim-container">
            <button
              @click="claimRewards"
              v-if="gameStore.game?.state === 'ongoing'"
              :disabled="gameStore.game?.clicks.length === 0"
            >
              claim ${{ prettify(gameStore.game?.pool) }}
            </button>
            <p
              v-if="gameStore.game?.state === 'done' && gameStore.game?.result === 'claimed'"
              style="color: var(--color-create-game-button)"
            >
              Claimed: ${{ prettify(gameStore.game?.pool) }}
              <AppCopyGameOutput />
            </p>
            <p
              v-if="gameStore.game?.state === 'done' && gameStore.game?.result === 'lost'"
              style="color: var(--color-bomb-background)"
            >
              You lost a pontential: ${{ prettify(gameStore.game?.pool) }}
              <AppCopyGameOutput />
            </p>
            <AppSkeletonLoader v-if="!gameStore.game" height="100%" />
          </div>
        </div>
      </div>
      <!-- <h1>minesweeper page</h1> -->
      <!-- <button @click="increase">up</button>
      <button @click="decrease">down</button>
      {{ gameStore.game }} -->
    </section>
  </main>
</template>

<style lang="scss" scoped>
main {
  flex-grow: 1;
  display: flex;
}

.big-container {
  width: 80%;
  height: 80%;
}
.game-container {
  height: 100%;
  aspect-ratio: 1/1;
  display: flex;
  justify-content: center;
  align-items: center;

  &::after {
    content: '';
    position: absolute;
    right: 0px;
    width: 2px;
    height: 90%;
    background: var(--color-container-titles);
  }
}

.control-container {
  height: 100%;
  flex-grow: 1;
  display: flex;
  justify-content: center;
  align-items: center;

  > div {
    height: 80%;
    width: 80%;
  }
}

.control-space {
  display: flex;
  flex-direction: column;
}

.infomatics-container {
  width: 100%;
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
}

.claim-container {
  height: 2rem;
  display: flex;
  justify-content: center;
  align-items: center;
  width: 80%;
  aspect-ratio: 8/1;
  min-height: 40px;
  border-radius: 5px;
  /* overflow: hidden; */
  margin-top: 20px;

  > * {
    text-transform: capitalize;
    border: none;
    width: 100%;
    display: flex;
    height: 100%;
    align-items: center;
    justify-content: center;
    font-weight: bold;
    font-size: 1rem;
  }

  button:not(:disabled) {
    background: var(--color-claim-button);
    color: var(--color-claim-button-text);

    &:hover {
      cursor: pointer;

      /* background:  */
    }
  }
  button:disabled {
    color: var(--color-claim-button);
    background: transparent;
    border: 2px solid var(--color-claim-button);
  }
}
</style>
