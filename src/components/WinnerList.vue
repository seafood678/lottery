<template>
  <div class="card">
    <div class="card-header">
      <h2 class="card-title">中奖名单</h2>
      <div class="header-actions" v-if="winners.length > 0">
        <button class="btn btn-outline btn-sm" @click="exportJson">
          <IconFileExport :size="16" />
          JSON
        </button>
        <button class="btn btn-outline btn-sm" @click="exportText">
          <IconFileText :size="16" />
          文本
        </button>
        <button class="btn btn-danger btn-sm" @click="clearAll">
          <IconTrash :size="16" />
          清空
        </button>
      </div>
    </div>

    <div class="winners-container" v-if="groupedWinners.length > 0">
      <div
        v-for="group in groupedWinners"
        :key="group.prizeId"
        class="prize-group fade-in"
      >
        <div class="prize-group-header">
          <span :class="getPrizeBadgeClass(group.prizeName)">
            {{ group.prizeName }}
          </span>
          <span class="winner-count">{{ group.winners.length }} 人</span>
        </div>
        <div class="prize-winners">
          <div
            v-for="winner in group.winners"
            :key="winner.id"
            class="winner-item"
          >
            <div class="winner-info">
              <span class="winner-name">{{ winner.person.name }}</span>
              <span class="winner-dept" v-if="winner.person.department">
                {{ winner.person.department }}
              </span>
            </div>
            <button
              class="btn btn-outline btn-sm revoke-btn"
              @click="revokeWinner(winner.id)"
              title="撤销"
            >
              <IconArrowBackUp :size="14" />
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="empty-state" v-else>
      <IconTrophy class="empty-state-icon" :size="48" />
      <div class="empty-state-text">暂无中奖记录</div>
    </div>

    <div class="modal-overlay" v-if="showExportModal" @click.self="showExportModal = false">
      <div class="modal">
        <div class="modal-header">
          <h3 class="modal-title">导出预览</h3>
          <button class="modal-close" @click="showExportModal = false">&times;</button>
        </div>
        <div class="modal-body">
          <textarea
            :value="exportContent"
            class="form-input export-textarea"
            rows="15"
            readonly
          ></textarea>
        </div>
        <div class="modal-footer">
          <button class="btn btn-outline" @click="showExportModal = false">关闭</button>
          <button class="btn btn-primary" @click="copyToClipboard">复制到剪贴板</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { IconTrophy, IconFileExport, IconFileText, IconTrash, IconArrowBackUp } from '@tabler/icons-vue'

const props = defineProps({
  winners: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['refresh'])

const showExportModal = ref(false)
const exportContent = ref('')

const groupedWinners = computed(() => {
  const groups = {}
  for (const winner of props.winners) {
    if (!groups[winner.prize_id]) {
      groups[winner.prize_id] = {
        prizeId: winner.prize_id,
        prizeName: winner.prize_name,
        winners: []
      }
    }
    groups[winner.prize_id].winners.push(winner)
  }
  return Object.values(groups)
})

const getPrizeBadgeClass = (prizeName) => {
  const name = prizeName.toLowerCase()
  if (name.includes('一等') || name.includes('特等') || name.includes('first')) {
    return 'badge badge-gold'
  } else if (name.includes('二等') || name.includes('second')) {
    return 'badge badge-silver'
  } else if (name.includes('三等') || name.includes('third')) {
    return 'badge badge-bronze'
  }
  return 'badge badge-default'
}

const revokeWinner = async (winnerId) => {
  if (!confirm('确定要撤销这条中奖记录吗？人员将被放回抽奖池。')) return

  try {
    await invoke('revoke_winner', { winnerId })
    emit('refresh')
  } catch (error) {
    alert('撤销失败: ' + error)
  }
}

const exportJson = async () => {
  try {
    const json = await invoke('export_winners_json')
    exportContent.value = json
    showExportModal.value = true
  } catch (error) {
    alert('导出失败: ' + error)
  }
}

const exportText = async () => {
  try {
    const text = await invoke('export_winners_text')
    exportContent.value = text
    showExportModal.value = true
  } catch (error) {
    alert('导出失败: ' + error)
  }
}

const clearAll = async () => {
  if (!confirm('确定要清空所有中奖记录吗？所有人员将被放回抽奖池。')) return

  try {
    await invoke('clear_winners')
    emit('refresh')
  } catch (error) {
    alert('清空失败: ' + error)
  }
}

const copyToClipboard = async () => {
  try {
    await navigator.clipboard.writeText(exportContent.value)
    alert('已复制到剪贴板')
  } catch (error) {
    const textarea = document.createElement('textarea')
    textarea.value = exportContent.value
    document.body.appendChild(textarea)
    textarea.select()
    document.execCommand('copy')
    document.body.removeChild(textarea)
    alert('已复制到剪贴板')
  }
}
</script>

<style scoped>
.header-actions {
  display: flex;
  gap: 8px;
}

.winners-container {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.prize-group {
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
  padding: 16px;
  border: 1px solid var(--card-border);
}

.prize-group-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--card-border);
}

.winner-count {
  color: var(--text-muted);
  font-size: 14px;
}

.prize-winners {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.winner-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 16px;
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  transition: all 0.3s ease;
}

.winner-item:hover {
  background: rgba(255, 255, 255, 0.08);
}

.winner-info {
  display: flex;
  flex-direction: column;
}

.winner-name {
  font-weight: 600;
}

.winner-dept {
  font-size: 12px;
  color: var(--text-muted);
}

.revoke-btn {
  opacity: 0;
  transition: opacity 0.3s ease;
}

.winner-item:hover .revoke-btn {
  opacity: 1;
}

.export-textarea {
  font-family: monospace;
  font-size: 12px;
  resize: none;
}
</style>
