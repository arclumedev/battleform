<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'
import { useMatchStore } from '../stores/match'

const matchStore = useMatchStore()
const logContainer = ref<HTMLElement | null>(null)

watch(
  () => matchStore.commandLog.length,
  async () => {
    await nextTick()
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  },
)
</script>

<template>
  <div class="command-log">
    <div class="log-header">
      <h3>Command Log</h3>
      <span class="log-count">{{ matchStore.commandLog.length }}</span>
    </div>
    <div ref="logContainer" class="log-entries">
      <div v-if="matchStore.commandLog.length === 0" class="log-empty">
        Waiting for commands...
      </div>
      <div
        v-for="(entry, i) in matchStore.commandLog"
        :key="i"
        class="log-entry"
      >
        <span class="entry-tick">T{{ entry.tick }}</span>
        <span class="entry-player">{{ entry.player }}</span>
        <span class="entry-action">{{ entry.action }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.command-log {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  border-top: 1px solid #1f2937;
}
.log-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0.75rem 1rem;
  border-bottom: 1px solid #1f2937;
}
.log-header h3 {
  font-size: 0.8125rem;
  font-weight: 600;
  color: #9ca3af;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
.log-count {
  font-size: 0.75rem;
  color: #4b5563;
  background: #1f2937;
  padding: 0.125rem 0.5rem;
  border-radius: 9999px;
}
.log-entries {
  flex: 1;
  overflow-y: auto;
  padding: 0.5rem;
}
.log-empty {
  color: #4b5563;
  font-size: 0.8125rem;
  text-align: center;
  padding: 2rem 1rem;
}
.log-entry {
  display: flex;
  align-items: baseline;
  gap: 0.5rem;
  padding: 0.25rem 0.5rem;
  font-size: 0.8125rem;
  font-family: 'SF Mono', 'Fira Code', 'Fira Mono', monospace;
  border-radius: 4px;
}
.log-entry:hover { background: #1f2937; }
.entry-tick {
  color: #4b5563;
  min-width: 3rem;
  flex-shrink: 0;
}
.entry-player {
  color: #60a5fa;
  font-weight: 500;
  flex-shrink: 0;
}
.entry-action {
  color: #9ca3af;
}
</style>
