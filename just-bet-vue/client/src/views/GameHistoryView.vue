<script lang="ts" setup>
import MineGameBoard from '~components/minesweeper/MineGameBoard.vue'
import MineBoardSquare from '~components/minesweeper/MineBoardSquare.vue'
import { hasBeenClicked } from '~services/hasBeenClicked'
import { getClickEarn } from '~services/getClickEarn'
import AppSkeletonLoader from '~components/AppSkeletonLoader.vue'
import { prettify } from '@/services/prettify'
import { useApi } from '~services/api'
import { onMounted, computed, ref, watch } from 'vue'

const { data, loading, get } = useApi('/api/game-history')

onMounted(() => {
  get()
})

const currentGame = ref<number>(0)

const games = computed(() => {
  return data.value ? data.value : undefined
})

const bigDisplayGame = computed(() => {
  return games.value ? games.value[currentGame.value] : undefined
})

const clicks = ref([])
const bombs = ref([])
const reference = ref(0)
const extraTimeout = ref()
const timeout = ref()

function updateCurrentGame(index: number) {
  if (currentGame.value === index) return
  currentGame.value = index
  clicks.value = []
  reference.value = 0
  bombs.value = []
  clearTimeout(extraTimeout.value)
  clearTimeout(timeout.value)
  showBomb()
}

function showBomb() {
  timeout.value = setTimeout(() => {
    if (bigDisplayGame.value && bigDisplayGame.value?.clicks.length > 0) {
      clicks.value.push(bigDisplayGame.value?.clicks[reference.value])
      reference.value += 1

      if (reference.value === bigDisplayGame.value?.clicks.length) {
        if (bigDisplayGame.value.result === 'claimed') {
          setTimeout(() => {
            bombs.value = bigDisplayGame.value.bomb.position
          }, 1000)
          // bombs.value = bigDisplayGame.value.bomb.position
        } else {
          bombs.value.push(bigDisplayGame.value?.clicks[reference.value - 1].position)
          setTimeout(() => {
            bombs.value = bigDisplayGame.value.bomb.position
          }, 1000)
        }
      } else {
        showBomb()
      }
    }
  }, 500)
}
showBomb()
</script>

<template>
  <main>
    <section class="game-container">
      <div class="big-container">
        <MineGameBoard :loading="loading">
          <MineBoardSquare
            v-for="index in bigDisplayGame?.size"
            :key="index"
            :id="index"
            :game="bigDisplayGame"
            :isBomb="bombs.includes(index)"
            :flip="hasBeenClicked(index, clicks)"
            :earn="getClickEarn(index, bigDisplayGame)"
          />
        </MineGameBoard>
      </div>
    </section>
    <section class="game-history">
      <div v-if="!loading">
        <div
          class="game"
          v-for="(game, index) in games"
          :key="index"
          @click="updateCurrentGame(index)"
          :class="{ active: index === currentGame }"
        >
          <div class="mini-container">
            <MineGameBoard :loading="loading">
              <MineBoardSquare
                v-for="index in game?.size"
                :key="index"
                :id="index"
                :game="game"
                :isBomb="game.bomb?.position?.includes(index)"
                :flip="hasBeenClicked(index, game.clicks)"
                :earn="getClickEarn(index, game)"
                :display-icon="false"
              />
            </MineGameBoard>
          </div>
          <div class="text-container">
            <h2>
              Game {{ games.length - index }}
              <span class="highlight" :class="game.result === 'claimed' ? 'win' : 'lose'"
                >{{ game.result }} ${{ prettify(game.pool) }}</span
              >
            </h2>
            <div>
              <p><span class="bold">stake:</span> ${{ prettify(game.stake) }}</p>
              <p><span class="bold">bomb count:</span> {{ game.bomb.count }}</p>
            </div>
          </div>
        </div>
      </div>
      <div class="skeletonHistoryLoader" v-else>
        <div class="game" v-for="index in 5" :key="index">
          <AppSkeletonLoader height="100%" />
          <p>loaking for games...</p>
        </div>
      </div>
    </section>
  </main>
</template>

<style lang="scss" scoped>
main {
  flex-grow: 1;
  display: flex;
}

.skeletonHistoryLoader p {
  position: absolute;
  top: 50%;
  color: var(--color-text-subtle);
  left: 50%;
  transform: translate(-50%, -50%);
}
.highlight {
  border-radius: 20px;
  font-weight: bold;
  font-size: 1rem;
  padding: 0.25rem 0.5rem;
  color: black;
  margin-left: auto;
}

.bold {
  font-weight: 900;
}
.win {
  background: var(--color-hightlight-green);
}

.lose {
  background: var(--color-bomb-background);
}
.text-container {
  padding: 10px 10px;
  color: var(--color-text-subtle);
  flex-grow: 1;
  display: flex;
  flex-direction: column;
  /* justify-content: space-between; */

  h2 {
    font-size: 1.25rem;
    display: flex;
    color: var(--color-text-title);
  }
}

.active {
  background: var(--color-container-titles);
  box-shadow: rgba(0, 0, 0, 0.24) 0px 3px 8px;
}

.big-container {
  width: 80%;
  height: 80%;
}

.mini-container {
  height: 100%;
  aspect-ratio: 1/1;

  > div {
    gap: 0rem !important;
  }
}

.game {
  /* background: red; */
  border-radius: 10px;
  border: 2px solid var(--color-container-titles);
  width: 100%;
  min-height: 100px;
  overflow: hidden;
  display: flex;

  &:hover {
    box-shadow: rgba(0, 0, 0, 0.24) 0px 3px 8px;
    cursor: pointer;
  }
}
.game-history {
  flex-grow: 1;
  display: flex;
  justify-content: center;
  align-items: center;

  > div {
    width: 80%;
    height: 80%;
    max-height: 417px;
    overflow-y: scroll;
    gap: 1rem;
    display: flex;
    flex-direction: column;
  }
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
</style>
