<script lang="ts" setup>
import { RouterView } from 'vue-router'
import AppCard from '~components/AppCard.vue'
import AppLogo from '~components/AppLogo.vue'
import AppClaim from '~components/AppClaim.vue'
import AppBalance from '~components/AppBalance.vue'
import AppFooter from '~components/AppFooter.vue'
import { Icons } from '~components/icons'
import { useRoute } from 'vue-router'
import { computed, watch } from 'vue'
import { useAuthStore } from '~stores/useAuthStore'
import { useApi } from '~services/api'
import { routerPathPrettify } from '@/services/routerPathPrettify'
import { socket } from '@/socket'
const { loading, get } = useApi('/api/get-user')

// disable sockets to prevent spam, as endpoint does not have the current url.
// socket.connect()

const authStore = useAuthStore()

watch(
  () => authStore.token,
  (to, from) => {
    get().then((response: { username: string; balance: number }) => {
      if (response) {
        authStore.username = response.username
        authStore.balance = response.balance
      }
    })
  }
)

const route = useRoute()

const heading = computed(() => {
  return routerPathPrettify(route.path)
})
</script>

<template>
  <div class="container">
    <div class="top-container">
      <AppLogo title="Just bet" />
      <AppClaim v-if="authStore.username" />
    </div>
    <AppCard :heading="heading">
      <template #icon>
        <component :is="Icons[heading.toLowerCase()]" />
      </template>
      <template #extra v-if="authStore.balance">
        <AppBalance :loading="loading" />
      </template>
      <RouterView />
    </AppCard>
    <footer v-if="authStore.username">
      <AppFooter :loading="loading" />
    </footer>
  </div>
</template>

<style lang="scss" scoped>
.top-container {
  display: flex;
  align-items: center;
  justify-content: space-between;

  > p {
    margin-right: 20px;
  }
}

footer {
  width: 100%;
  position: relative;
  display: flex;
  flex-grow: 1;
  padding: 50px 40px;
  display: flex;
  align-items: flex-start;
  color: var(--color-text-subtle);
  justify-content: space-between;
}
.container {
  min-width: 1000px;
  max-width: 100vh;
}
</style>
