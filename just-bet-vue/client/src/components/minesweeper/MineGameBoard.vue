<script lang="ts" setup>
import AppSkeletonLoader from '@/components/AppSkeletonLoader.vue'

const props = withDefaults(
  defineProps<{
    perRow?: number
    loading?: boolean
    message?: string
  }>(),
  {
    loading: false,
    perRow: 5,
    message: 'Looking for game...'
  }
)
</script>

<template>
  <div class="grid" v-if="!loading" :style="`--game-board-row-count: ${perRow}`">
    <slot />
  </div>
  <div class="skeleton-grid-loader" v-else>
    <AppSkeletonLoader height="100%" />
    <p>{{ message }}</p>
  </div>
</template>

<style lang="scss" scoped>
.skeleton-grid-loader,
.grid {
  width: 100%;
  height: 100%;
}

.skeleton-grid-loader {
  border-radius: 0.75rem;
  overflow: hidden;

  p {
    position: absolute;
    top: 50%;
    color: var(--color-text-subtle);
    left: 50%;
    transform: translate(-50%, -50%);
  }
}

.ready {
  &::after {
    content: 'user ready';
    position: absolute;
    text-transform: capitalize;
    padding: 0.5rem 1rem;
    width: calc(100% + 1rem);
    top: 50%;
    left: 50%;
    text-align: center;
    transform: translate(-50%, -50%);
    font-size: 1.2rem;
    background: rgba(0, 0, 0, 0.3);
  }
  &::before {
    content: '';
    position: absolute;
    width: calc(100% + 1rem);
    height: calc(100% + 1rem);
    left: -0.5rem;
    top: -0.5rem;
    // background: var(--color-card-main);
    background: var(--color-hightlight-green);
    opacity: 0.1;
    border-radius: 5px;
  }
}
.grid {
  display: grid;
  gap: 0.5rem;
  grid-template-columns: repeat(var(--game-board-row-count), 1fr);
  grid-template-rows: repeat(var(--game-board-row-count), 1fr);
}
</style>
