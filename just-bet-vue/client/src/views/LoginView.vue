<script lang="ts" setup>
import AppInput from '~components/AppInput.vue'
import { ref } from 'vue'

import { useRouter } from 'vue-router'
import { useAuthStore } from '~stores/useAuthStore'

const router = useRouter()
const authStore = useAuthStore()

const email = ref('')
const password = ref('')

const error = ref<undefined | string>(undefined)

async function handleLogin() {
  if (!email.value && !password.value) {
    error.value = 'must enter an email & password'
    return
  }
  if (!email.value) {
    error.value = 'must enter an email'
    return
  }
  if (!password.value) {
    error.value = 'must enter a password'
    return
  }
  error.value = undefined
  if (await authStore.login(email.value, password.value)) {
    router.push({ name: 'minesweeper' })
  }
}
</script>

<template>
  <div class="container">
    <form @submit.prevent="handleLogin" class="form">
      <AppInput
        label-input="email"
        text="Email"
        v-model="email"
        placeholder="Enter your email or username..."
        :showError="error?.includes('email')"
      ></AppInput>

      <AppInput
        label-input="password"
        type="password"
        text="Password"
        placeholder="Enter your password..."
        v-model="password"
        :showError="error?.includes('password')"
      ></AppInput>

      <!-- <p v-if="error" class="error">{{ error }}</p> -->
      <button class="login" type="submit">Login</button>
    </form>
    <section>
      <button class="register" @click="router.push({ name: 'register' })">Create an account</button>
    </section>
  </div>
</template>

<style lang="scss" scoped>
.container {
  display: flex;
  width: 100%;
  height: 100%;

  & > form,
  & > section {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 5rem;

    &:first-child::after {
      content: '';
      height: 80%;
      position: absolute;
      width: 2px;
      background: var(--color-container-titles);
      right: -1px;
    }
  }
}

.form {
  flex-direction: column;
  gap: 1rem;
}

button {
  border-radius: 10px;
  border: none;
  text-transform: capitalize;
  background: transparent;
  font-weight: bold;
  font-size: 1em;
}

.error {
  color: var(--color-text-subtle);
  text-transform: capitalize;
}
.login {
  padding: 0.5rem 1rem;
  border: 2px solid var(--color-create-game-button);
  color: var(--color-create-game-button);

  &:hover {
    cursor: pointer;
    background: var(--color-create-game-button);
    color: var(--color-create-game-button-text);
  }
}

.register {
  padding: 0.75rem 1.5rem;
  border: 2px solid var(--color-claim-button);
  color: var(--color-claim-button);

  &:hover {
    cursor: pointer;
    background: var(--color-claim-button);
    color: var(--color-claim-button-text);
  }
}
</style>
