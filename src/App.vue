<template>
  <div class="app" :class="{ 'fullscreen-active': isFullscreen }">
    <header class="app-header" v-if="!isFullscreen">
      <div class="title-wrapper" @click="startEditTitle">
        <h1 class="app-title" v-if="!isEditingTitle">{{ appTitle }}</h1>
        <input
          v-else
          ref="titleInput"
          v-model="appTitle"
          class="title-input"
          @blur="finishEditTitle"
          @keyup.enter="finishEditTitle"
        />
        <IconPencil class="edit-icon" :size="20" />
      </div>
      <div class="app-actions">
        <button class="btn btn-outline btn-sm" @click="soundEnabled = !soundEnabled">
          <IconVolume v-if="soundEnabled" :size="16" />
          <IconVolumeOff v-else :size="16" />
          {{ soundEnabled ? '声音开' : '声音关' }}
        </button>
        <button class="btn btn-outline btn-sm" @click="resetAllData">
          <IconRefresh :size="16" />
          重置数据
        </button>
      </div>
    </header>

    <nav class="tabs" v-if="!isFullscreen">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-btn"
        :class="{ active: currentTab === tab.id }"
        @click="currentTab = tab.id"
      >
        <component :is="tab.icon" :size="20" />
        <span class="tab-label">{{ tab.label }}</span>
      </button>
    </nav>

    <main class="app-main">
      <PersonManager
        v-show="currentTab === 'persons' && !isFullscreen"
        :persons="persons"
        :winners="winners"
        @refresh="loadAllData"
      />

      <PrizeManager
        v-show="currentTab === 'prizes' && !isFullscreen"
        :prizes="prizes"
        @refresh="loadAllData"
      />

      <LotteryWheel
        v-show="currentTab === 'lottery' || isFullscreen"
        :prizes="prizes"
        :persons="persons"
        :winners="winners"
        :app-title="appTitle"
        :sound-enabled="soundEnabled"
        @refresh="loadAllData"
        @fullscreenChange="handleFullscreenChange"
      />

      <WinnerList
        v-show="currentTab === 'winners' && !isFullscreen"
        :winners="winners"
        @refresh="loadAllData"
      />
    </main>
  </div>
</template>

<script setup>
import { ref, onMounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  IconDice5,
  IconUsers,
  IconGift,
  IconTrophy,
  IconPencil,
  IconRefresh,
  IconVolume,
  IconVolumeOff
} from '@tabler/icons-vue'
import PersonManager from './components/PersonManager.vue'
import PrizeManager from './components/PrizeManager.vue'
import LotteryWheel from './components/LotteryWheel.vue'
import WinnerList from './components/WinnerList.vue'

const currentTab = ref('lottery')
const isFullscreen = ref(false)
const persons = ref([])
const prizes = ref([])
const winners = ref([])
const appTitle = ref('年会抽奖系统')
const isEditingTitle = ref(false)
const titleInput = ref(null)
const soundEnabled = ref(true)

const tabs = [
  { id: 'lottery', label: '抽奖', icon: IconDice5 },
  { id: 'persons', label: '人员', icon: IconUsers },
  { id: 'prizes', label: '奖项', icon: IconGift },
  { id: 'winners', label: '中奖名单', icon: IconTrophy }
]

const startEditTitle = async () => {
  isEditingTitle.value = true
  await nextTick()
  titleInput.value?.focus()
  titleInput.value?.select()
}

const finishEditTitle = async () => {
  isEditingTitle.value = false
  if (!appTitle.value.trim()) {
    appTitle.value = '年会抽奖系统'
  }
  try {
    await invoke('set_app_title', { title: appTitle.value })
  } catch (error) {
    console.error('Failed to save title:', error)
  }
}

const loadAllData = async () => {
  try {
    const [personsData, prizesData, winnersData, titleData] = await Promise.all([
      invoke('get_all_persons'),
      invoke('get_all_prizes'),
      invoke('get_all_winners'),
      invoke('get_app_title')
    ])
    persons.value = personsData
    prizes.value = prizesData
    winners.value = winnersData
    if (titleData) {
      appTitle.value = titleData
    }
  } catch (error) {
    console.error('Failed to load data:', error)
  }
}

const handleFullscreenChange = (value) => {
  isFullscreen.value = value
}

const resetAllData = async () => {
  if (!confirm('确定要重置所有数据吗？此操作不可恢复！')) return

  try {
    await invoke('reset_all_data')
    await loadAllData()
    alert('数据已重置')
  } catch (error) {
    alert('重置失败: ' + error)
  }
}

onMounted(() => {
  loadAllData()
})
</script>

<style scoped>
.app {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  padding: 20px;
  max-width: 1400px;
  margin: 0 auto;
}

.app.fullscreen-active {
  padding: 0;
  max-width: none;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
  padding-bottom: 20px;
  border-bottom: 1px solid var(--card-border);
}

.title-wrapper {
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 8px;
  transition: background 0.3s;
}

.title-wrapper:hover {
  background: rgba(255, 255, 255, 0.05);
}

.title-wrapper:hover .edit-icon {
  opacity: 1;
}

.edit-icon {
  opacity: 0;
  color: var(--text-muted);
  transition: opacity 0.3s;
}

.app-title {
  font-size: 32px;
  font-weight: 700;
  background: linear-gradient(135deg, #e74c3c, #ff6b35, #f39c12);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.title-input {
  font-size: 32px;
  font-weight: 700;
  background: transparent;
  border: none;
  border-bottom: 2px solid var(--primary-color);
  color: var(--text-color);
  outline: none;
  padding: 0;
  width: 400px;
}

.app-actions {
  display: flex;
  gap: 12px;
}

.tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
  padding: 8px;
  background: var(--card-bg);
  border-radius: 12px;
  border: 1px solid var(--card-border);
}

.tab-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 20px;
  background: transparent;
  border: none;
  border-radius: 8px;
  color: var(--text-muted);
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.3s ease;
}

.tab-btn:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-color);
}

.tab-btn.active {
  background: linear-gradient(135deg, var(--primary-color), var(--primary-hover));
  color: white;
  box-shadow: 0 4px 15px rgba(231, 76, 60, 0.3);
}

.app-main {
  flex: 1;
}
</style>
