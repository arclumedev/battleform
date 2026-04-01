<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const auth = useAuthStore()
const router = useRouter()

const email = ref('')
const password = ref('')
const displayName = ref('')
const isRegister = ref(false)
const submitting = ref(false)

async function handleSubmit() {
  submitting.value = true
  try {
    if (isRegister.value) {
      await auth.register(email.value, password.value, displayName.value)
    } else {
      await auth.login(email.value, password.value)
    }
    router.push('/lobby')
  } catch {
    // error is set in the store
  } finally {
    submitting.value = false
  }
}

function oauthLogin(provider: string) {
  window.location.href = `/api/auth/${provider}`
}
</script>

<template>
  <div class="login-page">
    <div class="login-card">
      <h1 class="logo-text">BATTLEFORM</h1>
      <p class="tagline">Deterministic strategy. Real-time spectating.</p>

      <form @submit.prevent="handleSubmit" class="login-form">
        <div v-if="isRegister" class="field">
          <label for="displayName">Display Name</label>
          <input
            id="displayName"
            v-model="displayName"
            type="text"
            placeholder="Your display name"
            required
          />
        </div>
        <div class="field">
          <label for="email">Email</label>
          <input
            id="email"
            v-model="email"
            type="email"
            placeholder="you@example.com"
            required
          />
        </div>
        <div class="field">
          <label for="password">Password</label>
          <input
            id="password"
            v-model="password"
            type="password"
            placeholder="Password"
            required
          />
        </div>

        <p v-if="auth.error" class="error">{{ auth.error }}</p>

        <button type="submit" class="btn-primary" :disabled="submitting">
          {{ submitting ? 'Please wait...' : (isRegister ? 'Create Account' : 'Sign In') }}
        </button>

        <p class="toggle-mode">
          {{ isRegister ? 'Already have an account?' : "Don't have an account?" }}
          <a href="#" @click.prevent="isRegister = !isRegister">
            {{ isRegister ? 'Sign in' : 'Register' }}
          </a>
        </p>
      </form>

      <div class="divider">
        <span>or continue with</span>
      </div>

      <div class="oauth-buttons">
        <button class="btn-oauth" @click="oauthLogin('google')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92a5.06 5.06 0 0 1-2.2 3.32v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.1z"/><path d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/><path d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/><path d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/></svg>
          Google
        </button>
        <button class="btn-oauth" @click="oauthLogin('github')">
          <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"/></svg>
          GitHub
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 2rem;
}
.login-card {
  background: #111827;
  border: 1px solid #1f2937;
  border-radius: 12px;
  padding: 2.5rem;
  width: 100%;
  max-width: 420px;
}
.logo-text {
  font-size: 1.75rem;
  font-weight: 800;
  letter-spacing: 0.15em;
  text-align: center;
  background: linear-gradient(135deg, #60a5fa, #a78bfa);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  margin-bottom: 0.25rem;
}
.tagline {
  text-align: center;
  color: #6b7280;
  font-size: 0.875rem;
  margin-bottom: 2rem;
}
.login-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}
.field label {
  font-size: 0.8125rem;
  color: #9ca3af;
  font-weight: 500;
}
.field input {
  background: #0a0e17;
  border: 1px solid #374151;
  border-radius: 8px;
  padding: 0.625rem 0.875rem;
  color: #f9fafb;
  font-size: 0.9375rem;
  outline: none;
  transition: border-color 0.15s;
}
.field input:focus {
  border-color: #60a5fa;
}
.field input::placeholder {
  color: #4b5563;
}
.error {
  color: #f87171;
  font-size: 0.8125rem;
  text-align: center;
}
.btn-primary {
  background: linear-gradient(135deg, #3b82f6, #8b5cf6);
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 0.75rem;
  font-size: 0.9375rem;
  font-weight: 600;
  cursor: pointer;
  transition: opacity 0.15s;
}
.btn-primary:hover { opacity: 0.9; }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
.toggle-mode {
  text-align: center;
  font-size: 0.8125rem;
  color: #6b7280;
}
.toggle-mode a {
  color: #60a5fa;
  text-decoration: none;
}
.toggle-mode a:hover { text-decoration: underline; }
.divider {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin: 1.5rem 0;
  color: #4b5563;
  font-size: 0.8125rem;
}
.divider::before,
.divider::after {
  content: '';
  flex: 1;
  height: 1px;
  background: #1f2937;
}
.oauth-buttons {
  display: flex;
  gap: 0.75rem;
}
.btn-oauth {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.5rem;
  background: #1f2937;
  border: 1px solid #374151;
  border-radius: 8px;
  padding: 0.625rem;
  color: #d1d5db;
  font-size: 0.875rem;
  cursor: pointer;
  transition: background 0.15s;
}
.btn-oauth:hover { background: #374151; }
</style>
