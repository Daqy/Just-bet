<script lang="ts" setup>
import { Icons } from '~components/icons'
import { useGameStore } from '~stores/useGameStore'
import { prettify } from '@/services/prettify'
import { computed } from 'vue'
import { storeToRefs } from 'pinia'

const gameStore = useGameStore()
const { game } = storeToRefs(gameStore)

function getPercentageOfWining(size, nextClickCount, bombCount) {
  let total = 1
  for (let index = 0; index < nextClickCount; index++) {
    total *= (size - bombCount - index) / (size - index)
  }
  return total
}

const nextClick = computed(() => {
  if (!game.value) {
    return 0
  }
  const chanceOFWinning = getPercentageOfWining(
    game.value.size,
    game.value.clicks.length,
    game.value.bomb.count
  )
  const pool = (1 / chanceOFWinning) * game.value.stake // 25
  const potentialChance = getPercentageOfWining(
    game.value.size,
    game.value.clicks.length + 1,
    game.value.bomb.count
  )

  return (1 / potentialChance) * game.value.stake - pool
})
</script>

<template>
  <div class="informatics">
    <div class="information-container" title="Stake in game">
      <div class="svg-container">
        <component
          :is="Icons.coin"
          mainFill="var(--color-text-subtle)"
          secondaryFill="var(--color-container-titles)"
        />
      </div>
      <p>{{ prettify(game?.stake) }}</p>
    </div>
    <div class="information-container" title="Bomb Count">
      <div class="svg-container">
        <component :is="Icons.minesweeper" fill="var(--color-text-subtle)" />
      </div>
      <p>{{ game?.bomb.count || 0 }}</p>
    </div>
    <div class="information-container" title="Earn per click">
      <div class="svg-container">
        <component :is="Icons.mouseclick" fill="var(--color-text-subtle)" />
      </div>
      <p>{{ prettify(nextClick) }}</p>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.informatics {
  display: flex;
  width: 80%;
  justify-content: space-around;

  .information-container {
    border-radius: 5px;
    width: 25%;
    height: 96px;
    box-shadow: rgba(0, 0, 0, 0.24) 0px 3px 8px;
    background: var(--color-container-subtle);
  }

  p {
    height: 50%;
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
    font-size: 1rem;
    color: var(--color-text-subtle);
  }
}

.svg-container {
  width: 100%;
  height: 50%;
  box-shadow: rgba(0, 0, 0, 0.24) 0px 3px 8px;
  background: var(--color-container-titles);
  display: flex;
  justify-content: center;
  align-items: center;
  border-radius: 5px 5px 0px 0px;

  svg {
    height: 75%;
    aspect-ratio: 1/1;
  }
}
</style>
