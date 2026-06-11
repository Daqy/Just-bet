<script lang="ts" setup>
import { onUnmounted, computed, ref, watch, onMounted, nextTick } from 'vue'
import MineGameBoard from '~components/minesweeper/MineGameBoard.vue'
import BoardSquare from '~components/battleships/BoardSquare.vue'
import BattleshipShip from '~components/battleships/BattleshipShip.vue'
import { useApi } from '@/services/api'
import { game } from '@/mock/game'
import { state } from '@/socket'
import { useAuthStore } from '@/stores/useAuthStore'
import { useRoute, useRouter } from 'vue-router'
import { useSocketStore } from '@/stores/useSocketStore'

const authStore = useAuthStore()
const router = useRouter()

const socket = useSocketStore()

document.documentElement.style.setProperty('--margin-bottom-main-container', '50px')

onUnmounted(() => {
  document.documentElement.style.setProperty('--margin-bottom-main-container', '0px')
})

const gridRowCount = 8
const alphabet = 'abcdefghijklmnopqrstuvwxyz'

const { get, data, loading } = useApi('/api/battleships/latest-game')

get()

const lettersOnGrid = computed(() => {
  return alphabet.slice(0, gridRowCount).split('')
})

const grid = ref<HTMLDivElement>()
const fleet = ref<HTMLDivElement>()

const fleetPosition = computed(() => {
  return fleet.value?.getBoundingClientRect()
})

const createArray = (start: number, stop: number, step = 1) => {
  return Array.from({ length: stop - start }, (value, index) => start + index * step)
}

const shipFunctions = ref<{ [id: number]: any }>({})
const isHovering = ref<number[]>([])
const onGrid = ref<{ [id: number]: number[] }>(JSON.parse(localStorage.getItem('ships')) ?? {})

const getFunctionCalls = (data: { id: number; [fn: string]: any }) => {
  const { id, ...fn } = data
  shipFunctions.value[id] = fn

  if (!!onGrid.value[Number(id)]) {
    const gridBox =
      grid.value?.children[1].children[onGrid.value[id][0] - 1].getBoundingClientRect()
    fn.reposition({ x: gridBox?.x, y: gridBox?.y })
  }
}

const placeOnGrid = (id: number) => {
  if (isHovering.value.length === 0) {
    delete onGrid.value[id]
    shipFunctions.value[id].reset()
    localStorage.setItem('ships', JSON.stringify(onGrid.value))
    return
  }

  onGrid.value[id] = isHovering.value
  isHovering.value = []

  localStorage.setItem('ships', JSON.stringify(onGrid.value))

  const gridBox = grid.value?.children[1].children[onGrid.value[id][0] - 1].getBoundingClientRect()
  shipFunctions.value[id].reposition({ x: gridBox?.x, y: gridBox?.y })
}

const shipOnGrid = (gridNumber: number) => {
  if (!data.value) return false
  if (data.value.state === 'prep') {
    if (!onGrid.value) {
      return false
    }
    const keys = Object.keys(onGrid.value)
    for (const key of keys) {
      if (onGrid.value[Number(key)].includes(gridNumber)) {
        return true
      }
    }
    return false
  } else {
    const keys = Object.keys(data.value.ships)
    for (const key of keys) {
      if (data.value.ships[Number(key)].includes(gridNumber)) {
        return true
      }
    }
    return false
  }
}

const handleShipPositionUpdate = ({ id, position, isHorizontal }) => {
  if (!grid.value) {
    return
  }

  const gridPositions = grid.value.getBoundingClientRect()
  const padding = Number(window.getComputedStyle(grid.value).padding.replace(/\D+/g, ''))

  //check if ship is on the grid, if it isn't return
  if (
    !(
      position.x > gridPositions.x + padding &&
      gridPositions.x + gridPositions.width - padding > position.x &&
      position.y > gridPositions.y + padding &&
      gridPositions.y + gridPositions.height - padding > position.y
    )
  ) {
    isHovering.value = []

    return
  }
  // const gridSize = { size: 45, gap: 8 }
  const children = Array.from(grid.value?.children[1].children ?? [])

  children.forEach((gridBox, gridNumber) => {
    const gridBoxPosition = gridBox.getBoundingClientRect()
    // check if topleft of ship is in the box
    if (
      !(
        position.x > gridBoxPosition.x &&
        gridBoxPosition.x + gridBoxPosition.width > position.x &&
        position.y > gridBoxPosition.y &&
        gridBoxPosition.y + gridBoxPosition.height > position.y
      )
    ) {
      return
    }
    const hoverPosition = createArray(gridNumber + 1, gridNumber + 1 + id, isHorizontal ? 1 : 8)

    //check if the hover position is being placed onto a new line, if it prevent it.
    if (
      (isHorizontal &&
        Math.floor((hoverPosition[0] - 1) / 8) !==
          Math.floor((hoverPosition[hoverPosition.length - 1] - 1) / 8)) ||
      (!isHorizontal && Math.floor(hoverPosition[hoverPosition.length - 1]) / 8 > 8)
    ) {
      isHovering.value = []
      return
    }
    // check if ship is hovering another ship
    for (const pos of hoverPosition) {
      const keys = Object.keys(onGrid.value)

      for (const key of keys) {
        if (onGrid.value[Number(key)].includes(pos) && Number(key) !== id) {
          isHovering.value = []
          return
        }
      }
    }

    isHovering.value = hoverPosition
  })
}

const opponentHasClicked = (id: number) => {
  if (!data.value || data.value.state === 'awaiting') return false
  for (const key of Object.keys(data.value.opponent.clicks)) {
    if (Number(key) === id) {
      return true
    }
  }
  return false
}

const userHasClicked = (id: number) => {
  if (!data.value) return false
  for (const key of Object.keys(data.value.clicks)) {
    if (Number(key) === id) {
      return true
    }
  }

  if (data.value.state === 'done') {
    return true
  }
  return false
}

const userHasClickedBoat = (id: number) => {
  if (!data.value) return false
  for (const key of Object.keys(data.value.clicks)) {
    if (data.value.clicks[key][0] && Number(key) === id) {
      return true
    }
  }

  if (data.value.state === 'done') {
    for (const ship of Object.keys(data.value.ships)) {
      for (const position of data.value.opponent.ships[ship]) {
        if (position === id) {
          return true
        }
      }
    }
  }
  return false
}

const handleBoardClick = (id: number) => {
  if (!data.value) return
  if (!data.value.turn || data.value.state === 'done') return

  socket.emit('board-click', { id: data.value.id, clickNumber: id, token: authStore.token })

  // const { post } = useApi(`/api/game/battleships/${data.value.id}/click`)
  // post({ id }).then((response) => {
  //   if (response.state === 'done') {
  //     // game finished
  //   } else {

  //   }
  // })
}

const readyUp = () => {
  const confirmShipPlacement = useApi('/api/battleship/confirm-placement')

  confirmShipPlacement.post({ gameid: route.params.gameid, position: onGrid.value })
  localStorage.setItem('ships', JSON.stringify({}))
}

watch(
  () => state.game,
  () => {
    data.value = state.game
  }
)

watch(
  () => data.value,
  () => {
    if (data.value.ready) {
      onGrid.value = data.value.ships
      return
    }
  }
)

const route = useRoute()

socket.connect('/api/battleship/ws', route.params.gameid as string)
</script>

<template>
  <main class="game">
    <section class="player" ref="grid">
      <div class="gridCords">
        <div class="letters">
          <span v-for="(char, index) in lettersOnGrid" :key="index">{{ char }}</span>
        </div>
        <div class="numbers">
          <span v-for="number in gridRowCount" :key="number">{{ number }}</span>
        </div>
      </div>
      <!-- {{ botPositions }} {{ shipOnGrid }} -->
      <MineGameBoard :loading="loading" :perRow="gridRowCount">
        <BoardSquare
          v-for="gridNumber in 64"
          :key="gridNumber"
          :id="gridNumber"
          :flip="opponentHasClicked(gridNumber)"
          :hasBoat="shipOnGrid(gridNumber)"
          :userid="1"
          :class="{ glow: isHovering.includes(gridNumber) }"
        />
      </MineGameBoard>
    </section>

    <section class="opponent">
      <div class="gridCords">
        <div class="letters">
          <span v-for="(char, index) in lettersOnGrid" :key="index">{{ char }}</span>
        </div>
        <div class="num-right numbers">
          <span v-for="number in gridRowCount" :key="number">{{ number }}</span>
        </div>
      </div>
      <MineGameBoard
        :loading="!loading && data.state === 'awaiting'"
        :perRow="gridRowCount"
        message="awaiting player to join..."
        :class="{
          ready:
            !loading &&
            data.opponent &&
            !!data.opponent.ready &&
            data.state !== 'ongoing' &&
            data.state !== 'done'
        }"
      >
        <BoardSquare
          v-for="index in 64"
          :key="index"
          :id="index"
          :has-boat="userHasClickedBoat(index)"
          :flip="userHasClicked(index)"
          :userid="2"
          :game="data"
          @square-click="handleBoardClick(index)"
        />
      </MineGameBoard>
    </section>

    <div v-if="!loading && data.state === 'done'" class="test">
      <p
        :class="{
          win: data.winner === authStore.username,
          lose: data.winner !== authStore.username
        }"
      >
        {{ data.winner === authStore.username ? 'You won' : 'You lost' }} ${{ data.pool }}
      </p>
    </div>
    <section ref="fleet" class="fleet"></section>
    <div
      v-if="fleetPosition && !loading && data.state === 'prep' && !data.ready"
      class="ships"
      :style="`top: ${fleetPosition.y}px;left: ${fleetPosition.x}px;width: ${fleetPosition.width}px`"
    >
      <BattleshipShip
        v-for="(ship, index) in [5, 4, 3, 2, 1]"
        :key="ship"
        :id="ship"
        :position="{
          x: fleetPosition.x + 10 + (300 - 30 * (index - 1)) * index,
          y: fleetPosition.y + 10
        }"
        :has-been-placed="!!onGrid[ship]"
        @functions="getFunctionCalls"
        @position="handleShipPositionUpdate"
        @release="placeOnGrid"
      />

      <div class="button-container">
        <button v-if="Object.keys(onGrid).length === 5" class="button" @click="readyUp">
          Confirm Ship Placement
        </button>
      </div>
    </div>

    <div
      v-if="fleetPosition && !loading && data.ready"
      class="bottomSection"
      :style="`top: ${fleetPosition.y}px;left: ${fleetPosition.x}px;width: ${fleetPosition.width}px`"
    >
      <div
        v-if="!loading && data.ready && data.state !== 'ongoing' && data.state !== 'done'"
        class="ready-text"
      >
        <p>you are ready</p>
      </div>
      <div class="ready-text" v-if="!loading && data.state === 'done'">
        <button class="button" @click="router.push('/battleships')">Back to games</button>
      </div>
      <div class="ready-text" v-if="!loading && data.state === 'ongoing'">
        <p>{{ data.turn ? 'Your go to select tile' : 'opponents go to select tile' }}</p>
      </div>
    </div>
  </main>
</template>

<style lang="scss" scoped>
.bottomSection {
  position: fixed;
}

.test {
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  justify-content: center;
  align-items: center;
  position: absolute;
  height: 100%;
  width: 100%;
  border-radius: 0 0 10px 10px;

  text-transform: capitalize;
  .win {
    color: var(--color-hightlight-green);
  }

  .lose {
    color: var(--color-bomb-background);
  }
}

.game {
  position: relative;
  display: flex;
  flex-grow: 1;

  & > section:not(.fleet) {
    user-select: none;

    /* width: 50%; */
    padding: 40px;
    height: calc(100% - 22px);
    aspect-ratio: 1/1;
  }
}

.ships {
  position: fixed;
}

.ships,
.ready-text {
  display: flex;
  /* justify-content: space-between; */
  gap: 1.85rem;
  padding: 10px;
  /* justify-items: space-between; */
  /* justify-items: */
}

.ready-text {
  height: 65.5px;
  display: flex;
  justify-content: center;
  align-items: center;
  text-transform: capitalize;
}

.fleet {
  position: absolute;
  width: 90%;
  height: 65.5px;
  background: var(--color-container-titles);
  box-shadow: rgba(0, 0, 0, 0.1) 0px 4px 12px;
  bottom: -44px;
  border-radius: 10px;
  left: 50%;
  transform: translate(-50%, 0);
}

.button-container {
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
}

.button {
  text-transform: capitalize;
}

.button:not(:disabled) {
  background: var(--color-container-subtle);
  box-shadow: rgba(0, 0, 0, 0.1) 0px 4px 12px;
  border: 2px solid var(--color-create-game-button);
  border-radius: 5px;
  min-height: 45.5px;
  padding: 10px 20px;
  /* aspect-ratio: 4/1; */
  font-size: 1em;
  font-weight: bold;
  color: var(--color-create-game-button);

  &:hover {
    cursor: pointer;
    background: var(--color-create-game-button);
    color: var(--color-create-game-button-text);
  }
}

.gridCords {
  position: absolute;
  top: 0px;
  left: 0px;
  color: var(--color-text-subtle);
  .letters {
    position: absolute;
    top: 20px;
    left: 40px;
    transform: translate(0, -50%);
    text-transform: uppercase;
    display: flex;
    gap: 0.5rem;
    user-select: none;

    /* gap: 0 20px; */

    span {
      display: flex;
      justify-content: center;
      width: calc(calc(428px - calc(0.5rem * 8)) / 8);
    }
  }

  .numbers {
    position: absolute;
    user-select: none;
    top: 40px;
    left: 20px;
    transform: translate(-50%, 0);
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    /* gap: 0 20px; */

    span {
      display: flex;
      align-items: center;
      height: calc(calc(428px - calc(0.5rem * 8)) / 8);
    }
  }
}

.num-right {
  left: 480px;
}

.player {
  position: relative;
  /* background: blue; */
  &::after {
    content: '';
    position: absolute;
    top: 5%;
    right: 0px;
    width: 2px;
    height: 90%;
    background: var(--color-container-titles);
  }
}
.opponent {
  /* background: red; */
}
</style>
