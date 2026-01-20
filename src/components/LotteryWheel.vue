<template>
  <div class="lottery-container" :class="{ 'fullscreen-mode': isFullscreen }">
    <div class="lottery-header" v-if="!isFullscreen">
      <div></div>
      <button class="btn btn-outline" @click="toggleFullscreen">
        <IconMaximize :size="16" />
        全屏展示
      </button>
    </div>

    <div class="lottery-main">
      <div class="prize-selector" v-if="!isFullscreen">
        <label class="form-label">选择奖项:</label>
        <select v-model="selectedPrizeId" class="form-input prize-select">
          <option value="">-- 请选择奖项 --</option>
          <option
            v-for="prize in availablePrizes"
            :key="prize.id"
            :value="prize.id"
            :disabled="prize.remaining_count === 0"
          >
            {{ prize.name }} - {{ prize.description }} (剩余: {{ prize.remaining_count }})
          </option>
        </select>
      </div>

      <div class="draw-count-selector" v-if="!isFullscreen && selectedPrize">
        <label class="form-label">抽取数量:</label>
        <div class="count-buttons">
          <button
            v-for="n in Math.min(5, selectedPrize.remaining_count)"
            :key="n"
            class="btn"
            :class="drawCount === n ? 'btn-primary' : 'btn-outline'"
            @click="drawCount = n"
          >
            {{ n }}
          </button>
        </div>
      </div>

      <h1 class="fullscreen-title" v-if="isFullscreen">{{ appTitle }}</h1>

      <div class="display-area">
        <div class="fullscreen-prize-info" v-if="isFullscreen && selectedPrize">
          <span class="prize-badge">{{ selectedPrize.name }}</span>
          <span class="prize-desc">{{ selectedPrize.description }}</span>
        </div>

        <div class="name-display" :class="{ rolling: isRolling, winner: showWinner }">
          <div class="name-window">
            <transition-group name="roll" tag="div" class="name-list">
              <div
                v-for="name in displayNames"
                :key="name.key"
                class="name-item"
              >
                {{ name.text }}
              </div>
            </transition-group>
          </div>
        </div>

        <div class="winner-announcement" v-if="showWinner && lastWinners.length > 0">
          <div class="winner-label">恭喜中奖!</div>
          <div class="winners-list">
            <div v-for="winner in lastWinners" :key="winner.id" class="winner-name">
              {{ winner.person.name }}
              <span v-if="winner.person.department" class="winner-dept">
                ({{ winner.person.department }})
              </span>
            </div>
          </div>
        </div>
      </div>

      <div class="control-buttons">
        <button
          v-if="!isRolling && !showWinner"
          class="btn btn-primary btn-lg start-btn"
          @click="startRolling"
          :disabled="!canStart"
        >
          <IconPlayerPlay :size="24" />
          开始抽奖
        </button>
        <button
          v-if="isRolling"
          class="btn btn-danger btn-lg stop-btn"
          @click="stopRolling"
        >
          <IconPlayerStop :size="24" />
          停止
        </button>
        <button
          v-if="showWinner"
          class="btn btn-secondary btn-lg"
          @click="resetDisplay"
        >
          <IconRefresh :size="24" />
          继续抽奖
        </button>
      </div>

      <button
        v-if="isFullscreen"
        class="btn btn-outline fullscreen-exit"
        @click="toggleFullscreen"
      >
        <IconMinimize :size="16" />
        退出全屏 (ESC)
      </button>
    </div>

    <div class="confetti-container" v-if="showConfetti">
      <div
        v-for="i in 100"
        :key="i"
        class="confetti"
        :class="{ fountain: confettiStyles[i - 1]?.isFountain }"
        :style="confettiStyles[i - 1]"
      ></div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import {
  IconMaximize,
  IconMinimize,
  IconPlayerPlay,
  IconPlayerStop,
  IconRefresh
} from '@tabler/icons-vue'

const props = defineProps({
  prizes: {
    type: Array,
    default: () => []
  },
  persons: {
    type: Array,
    default: () => []
  },
  winners: {
    type: Array,
    default: () => []
  },
  appTitle: {
    type: String,
    default: '年会抽奖系统'
  },
  soundEnabled: {
    type: Boolean,
    default: true
  }
})

const startSound = new Audio('/sounds/start.wav')
const winSound = new Audio('/sounds/win.mp3')

const playStartSound = () => {
  if (props.soundEnabled) {
    startSound.currentTime = 0
    startSound.play().catch(() => {})
  }
}

const playWinSound = () => {
  startSound.pause()
  startSound.currentTime = 0

  if (props.soundEnabled) {
    winSound.currentTime = 0
    winSound.play().catch(() => {})
  }
}

const emit = defineEmits(['refresh', 'fullscreenChange'])

const selectedPrizeId = ref('')
const drawCount = ref(1)
const isRolling = ref(false)
const showWinner = ref(false)
const showConfetti = ref(false)
const confettiStyles = ref([])
const isFullscreen = ref(false)
const lastWinners = ref([])
const displayNames = ref([{ key: 0, text: '准备开始' }])
const rollInterval = ref(null)
const nameKey = ref(0)

const availablePrizes = computed(() => {
  return props.prizes.filter(p => p.remaining_count > 0)
})

const selectedPrize = computed(() => {
  return props.prizes.find(p => p.id === selectedPrizeId.value)
})

const availablePersons = computed(() => {
  const winnerIds = props.winners.map(w => w.person.id)
  return props.persons.filter(p => !winnerIds.includes(p.id))
})

const canStart = computed(() => {
  return selectedPrize.value &&
    selectedPrize.value.remaining_count >= drawCount.value &&
    availablePersons.value.length >= drawCount.value
})

watch(selectedPrizeId, () => {
  drawCount.value = 1
  showWinner.value = false
  lastWinners.value = []
})

const startRolling = () => {
  if (!canStart.value) return
  isRolling.value = true
  showWinner.value = false
  showConfetti.value = false
  lastWinners.value = []

  playStartSound()

  rollInterval.value = setInterval(() => {
    const randomNames = []
    for (let i = 0; i < drawCount.value; i++) {
      const randomPerson = availablePersons.value[
        Math.floor(Math.random() * availablePersons.value.length)
      ]
      if (randomPerson) {
        randomNames.push({ key: ++nameKey.value, text: randomPerson.name })
      }
    }
    if (randomNames.length > 0) {
      displayNames.value = randomNames
    }
  }, 50)
}

const stopRolling = async () => {
  clearInterval(rollInterval.value)
  isRolling.value = false

  try {
    const winners = await invoke('draw_lottery', {
      prizeId: selectedPrizeId.value,
      count: drawCount.value
    })

    lastWinners.value = winners
    displayNames.value = winners.map((w, i) => ({
      key: ++nameKey.value,
      text: w.person.name
    }))

    showWinner.value = true
    generateConfetti()
    showConfetti.value = true
    playWinSound()

    setTimeout(() => {
      showConfetti.value = false
    }, 4000)

    emit('refresh')
  } catch (error) {
    alert('抽奖失败: ' + error)
    displayNames.value = [{ key: 0, text: '准备开始' }]
  }
}

const resetDisplay = () => {
  showWinner.value = false
  lastWinners.value = []
  displayNames.value = [{ key: 0, text: '准备开始' }]
}

const toggleFullscreen = () => {
  isFullscreen.value = !isFullscreen.value
  emit('fullscreenChange', isFullscreen.value)
}

const handleKeydown = (e) => {
  const tagName = e.target.tagName.toLowerCase()
  if (tagName === 'input' || tagName === 'textarea') {
    return
  }

  if (e.key === 'Escape' && isFullscreen.value) {
    toggleFullscreen()
  }
  if (e.key === ' ' || e.key === 'Enter') {
    e.preventDefault()
    if (isRolling.value) {
      stopRolling()
    } else if (!showWinner.value && canStart.value) {
      startRolling()
    } else if (showWinner.value) {
      resetDisplay()
    }
  }
}

const generateConfetti = () => {
  const styles = []
  const colors = ['#e74c3c', '#3498db', '#27ae60', '#f39c12', '#9b59b6', '#ffd700', '#ff69b4', '#00ffff']

  for (let i = 0; i < 100; i++) {
    const color = colors[i % colors.length]
    const size = Math.random() * 12 + 6
    const delay = Math.random() * 0.2
    const rotation = Math.random() * 1080 - 540

    const isFountain = Math.random() < 0.65

    if (isFountain) {
      const initialVx = (Math.random() - 0.5) * 800
      const initialVy = -500 - Math.random() * 600
      const gravity = 1200 + Math.random() * 400

      const peakT = 0.35
      const endT = 1.0

      const midX = initialVx * peakT
      const midY = initialVy * peakT + 0.5 * gravity * peakT * peakT

      const endX = initialVx * endT
      const endY = initialVy * endT + 0.5 * gravity * endT * endT

      styles.push({
        '--mid-x': `${midX}px`,
        '--mid-y': `${midY}px`,
        '--end-x': `${endX}px`,
        '--end-y': `${endY}px`,
        '--rotation': `${rotation}deg`,
        width: `${size}px`,
        height: `${size}px`,
        background: color,
        animationDelay: `${delay}s`,
        borderRadius: Math.random() > 0.5 ? '50%' : '2px',
        isFountain: true
      })
    } else {
      const angle = Math.random() * 360
      const velocity = 2000 + Math.random() * 2500
      const radians = (angle * Math.PI) / 180
      const tx = Math.cos(radians) * velocity
      const ty = Math.sin(radians) * velocity

      styles.push({
        '--tx': `${tx}px`,
        '--ty': `${ty}px`,
        '--rotation': `${rotation}deg`,
        width: `${size}px`,
        height: `${size}px`,
        background: color,
        animationDelay: `${delay * 0.5}s`,
        borderRadius: Math.random() > 0.5 ? '50%' : '2px',
        isFountain: false
      })
    }
  }

  confettiStyles.value = styles
}

onMounted(() => {
  window.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown)
  clearInterval(rollInterval.value)
})
</script>

<style scoped>
.lottery-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.lottery-container.fullscreen-mode {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: linear-gradient(135deg, #1a1a2e 0%, #0f0f23 100%);
  z-index: 1000;
  padding: 40px;
}

.lottery-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 30px;
}

.lottery-title {
  font-size: 28px;
  font-weight: 700;
  background: linear-gradient(135deg, var(--gold), var(--primary-color));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.fullscreen-title {
  font-size: 48px;
  font-weight: 700;
  text-align: center;
  background: linear-gradient(135deg, #e74c3c, #ff6b35, #f39c12);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin-bottom: 20px;
}

.lottery-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 30px;
}

.prize-selector {
  display: flex;
  align-items: center;
  gap: 16px;
}

.prize-selector .form-label {
  white-space: nowrap;
  margin-bottom: 0;
}

.prize-select {
  min-width: 350px;
}

.draw-count-selector {
  display: flex;
  align-items: center;
  gap: 16px;
}

.count-buttons {
  display: flex;
  gap: 8px;
}

.display-area {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.fullscreen-prize-info {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  margin-bottom: 20px;
}

.fullscreen-prize-info .prize-badge {
  font-size: 32px;
  padding: 10px 30px;
  background: linear-gradient(135deg, var(--gold), #ffa500);
  color: #333;
  border-radius: 30px;
  font-weight: 700;
}

.fullscreen-prize-info .prize-desc {
  font-size: 24px;
  color: var(--text-muted);
}

.name-display {
  background: linear-gradient(135deg, rgba(255,255,255,0.1), rgba(255,255,255,0.05));
  border: 3px solid var(--card-border);
  border-radius: 20px;
  padding: 40px 80px;
  min-width: 400px;
  text-align: center;
  transition: all 0.3s ease;
}

.fullscreen-mode .name-display {
  min-width: 600px;
  padding: 60px 120px;
}

.name-display.rolling {
  border-color: var(--secondary-color);
  box-shadow: 0 0 30px rgba(52, 152, 219, 0.5);
}

.name-display.winner {
  border-color: var(--gold);
  box-shadow: 0 0 50px rgba(255, 215, 0, 0.5);
  animation: pulse 0.5s ease infinite;
}

.name-window {
  overflow: hidden;
  height: 80px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.fullscreen-mode .name-window {
  height: 120px;
}

.name-list {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
}

.name-item {
  font-size: 48px;
  font-weight: 700;
  color: var(--text-color);
  white-space: nowrap;
}

.fullscreen-mode .name-item {
  font-size: 72px;
}

.winner-announcement {
  text-align: center;
  animation: fadeIn 0.5s ease;
}

.winner-label {
  font-size: 24px;
  color: var(--gold);
  font-weight: 700;
  margin-bottom: 16px;
}

.fullscreen-mode .winner-label {
  font-size: 36px;
}

.winners-list {
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 20px;
}

.winner-name {
  font-size: 28px;
  color: var(--text-color);
  padding: 10px 24px;
  background: rgba(255, 215, 0, 0.2);
  border-radius: 10px;
  border: 2px solid var(--gold);
}

.fullscreen-mode .winner-name {
  font-size: 36px;
}

.winner-dept {
  font-size: 18px;
  color: var(--text-muted);
}

.fullscreen-mode .winner-dept {
  font-size: 24px;
}

.control-buttons {
  display: flex;
  gap: 20px;
}

.start-btn {
  min-width: 200px;
  font-size: 20px;
  padding: 20px 60px;
}

.fullscreen-mode .start-btn {
  font-size: 28px;
  padding: 25px 80px;
}

.stop-btn {
  animation: pulse 0.5s ease infinite;
}

.fullscreen-exit {
  position: absolute;
  bottom: 30px;
  right: 30px;
}

.roll-enter-active {
  transition: all 0.05s ease;
}

.roll-leave-active {
  transition: all 0.05s ease;
}

.roll-enter-from {
  opacity: 0;
  transform: translateY(-20px);
}

.roll-leave-to {
  opacity: 0;
  transform: translateY(20px);
}
</style>
