<script lang="ts" setup>
import { useAuthStore } from '~stores/useAuthStore'
import { useRouter } from 'vue-router'
import { useApi } from '@/services/api'
import AppSkeletonLoader from './AppSkeletonLoader.vue'

const props = withDefaults(
  defineProps<{
    loading?: boolean
  }>(),
  {
    loading: false
  }
)

const authStore = useAuthStore()
const router = useRouter()

function logout() {
  const logout = useApi('/api/logout')

  logout
    .post({ token: authStore.token })
    .then((response: any) => {
      if (response) {
        authStore._reset()
      }
    })
    .finally(() => router.push({ name: 'login' }))
}
</script>

<template>
  <span v-if="!loading">
    Username: {{ authStore.username }} | <button class="logout" @click="logout">logout</button>
  </span>

  <p class="copyright" v-if="!loading">© 2023 Daqy Develops & Co</p>
  <AppSkeletonLoader height="2rem" v-else />
</template>

<style lang="scss" scoped>
/* footer {
  width: 100%;
  display: flex;
  flex-grow: 1;
  padding: 50px 40px;
  display: flex;
  align-items: flex-start;
  color: var(--color-text-subtle);
  justify-content: space-between;
} */

.logout {
  border: none;
  background: none;
  font-size: 1rem;
  font-weight: bold;
  color: var(--color-text-subtle);

  &:hover {
    cursor: pointer;
    color: var(--color-text-subtle-600);
    text-decoration: underline;
  }
}
</style>
