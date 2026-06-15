<script lang="ts" setup>
import { ref } from 'vue'
import AppInput from '~components/AppInput.vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '~stores/useAuthStore'
import { useApi } from '@/services/api'

const router = useRouter()
const authStore = useAuthStore()

const username = ref('')
const email = ref('')
const password = ref('')
const confirm = ref('')
const cookies = ref(false)
const terms = ref(false)
const openModal = ref(false)

const error = ref<undefined | string>(undefined)

function handleRegister() {
  if (
    username.value === '' &&
    email.value === '' &&
    password.value === '' &&
    confirm.value === ''
  ) {
    error.value = 'please enter your username, email, password and re-confirm'

    return
  }
  if (username.value === '') {
    error.value = 'must enter a username'

    return
  }

  if (email.value === '') {
    error.value = 'must enter an email'

    return
  }

  if (password.value === '') {
    error.value = 'must enter a password'

    return
  }
  if (confirm.value === '') {
    error.value = 'must enter re-confirm'

    return
  }

  if (!cookies.value) {
    error.value = 'must agree to cookies'

    return
  }

  if (!terms.value) {
    error.value = 'must agree to terms'

    return
  }

  if (password.value !== confirm.value) {
    return
  }

  const { post } = useApi('/api/register')
  post({ username: username.value, password: password.value, email: email.value })
    .then((response: { token: string }) => {
      authStore.token = response.token
    })
    .finally(() => {
      router.push({ name: 'minesweeper' })
    })
}
</script>

<template>
  <div class="container">
    <section>
      <button class="login" @click="router.push({ name: 'login' })">Login to your account</button>
    </section>
    <section class="form">
      <AppInput
        label-input="username"
        type="text"
        text="Username"
        v-model="username"
        placeholder="Enter your username..."
        :showError="error?.includes('username')"
      ></AppInput>
      <AppInput
        label-input="email"
        type="email"
        text="Email"
        v-model="email"
        placeholder="Enter your email..."
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
      <AppInput
        label-input="confirm"
        type="password"
        text="Confirm Password"
        placeholder="Enter your password..."
        v-model="confirm"
        :showError="error?.includes('confirm')"
      ></AppInput>
      <div class="terms">
        <label for="terms">
          <input type="checkbox" v-model="terms" />
          <span
            >Do you agree to the
            <span class="important" @click="openModal = true">terms & condtions</span></span
          >
        </label>
        <label for="cookies">
          <input type="checkbox" v-model="cookies" />
          <span>Do you agree to us using cookies</span>
        </label>
      </div>
      <!-- <p v-if="error" class="error">{{ error }}</p> -->
      <button class="register" @click="handleRegister">Register</button>
    </section>
  </div>

  <div class="termsModal" v-if="openModal">
    <div class="card">
      <h1>Terms & Conditions</h1>
      <section>
        <h2>1. Data</h2>
        <p>
          1.1. We are not responsible for any data leaks as this is not secure and you should not
          use any passwords to any other website
        </p>
        <p>1.2. We do not verify any emails as of 11/06/2023</p>
      </section>
      <section>
        <h2>2. Random</h2>
        <p>
          2.1 This website was creating for fun, you cannot cash out or do anything with the money
          you earn on the website. So just play and enjoy your time :)
        </p>
      </section>
      <div class="buttons">
        <button @click="openModal = false" class="close">Close</button>
        <button
          class="accept"
          @click="
            () => {
              openModal = false
              terms = true
            }
          "
        >
          Accept terms
        </button>
      </div>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.error {
  color: var(--color-text-subtle);
  text-transform: capitalize;
}
.termsModal {
  position: fixed;
  top: 0px;
  left: 0px;
  width: 100%;
  height: 100%;
  display: flex;
  justify-content: center;
  align-items: center;
  background: rgba(0, 0, 0, 0.5);
  .card {
    padding: 50px;
    border-radius: 10px;
    background: var(--color-background);
    max-width: 80%;

    .buttons {
      display: flex;
      margin-top: 20px;
      justify-content: space-between;
    }

    button {
      text-transform: capitalize;
      padding: 0.5rem 1rem;
      color: var(--color-text-title);

      &:hover {
        cursor: pointer;
      }
    }

    .close {
      border: 2px solid var(--color-claim-button);
      color: var(--color-claim-button);

      &:hover {
        background: var(--color-claim-button);
        color: var(--color-claim-button-text);
      }
    }

    .accept {
      border: 2px solid var(--color-create-game-button);
      color: var(--color-create-game-button);

      &:hover {
        background: var(--color-create-game-button);
        color: var(--color-create-game-button-text);
      }
    }

    h1 {
      margin-bottom: 15px;
    }

    section {
      margin-bottom: 10px;
    }

    p {
      margin-bottom: 5px;
      color: var(--color-text-subtle);
    }
  }
}
.terms {
  border-top: 2px solid var(--color-container-titles);
  padding-top: 10px;
  display: flex;
  flex-direction: column;

  label {
    color: var(--color-text-subtle);

    .important {
      color: var(--color-text-title);

      &:hover {
        cursor: pointer;
      }
    }
  }

  input {
    margin-right: 10px;
  }
}
.container {
  display: flex;
  width: 100%;
  height: 100%;

  & > section {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    padding: 0 5rem;

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
.register {
  padding: 0.5rem 1rem;
  border: 2px solid var(--color-create-game-button);
  color: var(--color-create-game-button);

  &:hover {
    cursor: pointer;
    background: var(--color-create-game-button);
    color: var(--color-create-game-button-text);
  }
}

.login {
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
