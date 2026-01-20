<template>
  <div class="card">
    <div class="card-header">
      <h2 class="card-title">奖项设置</h2>
    </div>

    <form @submit.prevent="addPrize" class="add-form">
      <div class="form-row">
        <div class="form-group">
          <input
            v-model="newPrize.name"
            type="text"
            class="form-input"
            placeholder="奖项名称 (如: 一等奖) *"
            required
          />
        </div>
        <div class="form-group">
          <input
            v-model="newPrize.description"
            type="text"
            class="form-input"
            placeholder="奖品描述 (如: iPhone 16) *"
            required
          />
        </div>
        <div class="form-group" style="max-width: 100px;">
          <input
            v-model.number="newPrize.count"
            type="number"
            min="1"
            class="form-input"
            placeholder="数量 *"
            required
          />
        </div>
        <button type="submit" class="btn btn-primary" :disabled="!canAddPrize">
          <IconPlus :size="16" />
          添加
        </button>
      </div>
    </form>

    <div class="prize-list" v-if="prizes.length > 0">
      <div
        v-for="prize in prizes"
        :key="prize.id"
        class="prize-item fade-in"
        :class="{ 'prize-exhausted': prize.remaining_count === 0 }"
      >
        <div class="prize-info">
          <div class="prize-name">
            <span :class="getPrizeBadgeClass(prize)">{{ prize.name }}</span>
          </div>
          <div class="prize-desc">{{ prize.description }}</div>
          <div class="prize-count">
            剩余: <strong>{{ prize.remaining_count }}</strong> / {{ prize.total_count }}
          </div>
        </div>
        <div class="prize-actions">
          <button
            class="btn btn-outline btn-sm"
            @click="openEditModal(prize)"
          >
            <IconEdit :size="14" />
          </button>
          <button
            class="btn btn-danger btn-sm"
            @click="removePrize(prize.id)"
            :disabled="prize.total_count !== prize.remaining_count"
          >
            <IconTrash :size="14" />
          </button>
        </div>
      </div>
    </div>

    <div class="empty-state" v-else>
      <IconGift class="empty-state-icon" :size="48" />
      <div class="empty-state-text">暂无奖项，请添加抽奖奖项</div>
    </div>

    <div class="modal-overlay" v-if="showEditModal" @click.self="showEditModal = false">
      <div class="modal">
        <div class="modal-header">
          <h3 class="modal-title">编辑奖项</h3>
          <button class="modal-close" @click="showEditModal = false">&times;</button>
        </div>
        <div class="modal-body">
          <div class="form-group">
            <label class="form-label">奖项名称</label>
            <input
              v-model="editingPrize.name"
              type="text"
              class="form-input"
              required
            />
          </div>
          <div class="form-group">
            <label class="form-label">奖品描述</label>
            <input
              v-model="editingPrize.description"
              type="text"
              class="form-input"
              required
            />
          </div>
          <div class="form-group">
            <label class="form-label">奖项数量</label>
            <input
              v-model.number="editingPrize.count"
              type="number"
              :min="editingPrize.usedCount"
              class="form-input"
              required
            />
            <p style="color: var(--text-muted); font-size: 12px; margin-top: 8px;" v-if="editingPrize.usedCount > 0">
              已抽出 {{ editingPrize.usedCount }} 个，数量不能少于此值
            </p>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn btn-outline" @click="showEditModal = false">取消</button>
          <button class="btn btn-primary" @click="updatePrize">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { IconGift, IconPlus, IconEdit, IconTrash } from '@tabler/icons-vue'

const props = defineProps({
  prizes: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['refresh'])

const newPrize = ref({
  name: '',
  description: '',
  count: 1
})

const showEditModal = ref(false)
const editingPrize = ref({
  id: '',
  name: '',
  description: '',
  count: 1,
  usedCount: 0
})

const canAddPrize = computed(() => {
  return newPrize.value.name && newPrize.value.description && newPrize.value.count > 0
})

const getPrizeBadgeClass = (prize) => {
  const name = prize.name.toLowerCase()
  if (name.includes('一等') || name.includes('特等') || name.includes('first')) {
    return 'badge badge-gold'
  } else if (name.includes('二等') || name.includes('second')) {
    return 'badge badge-silver'
  } else if (name.includes('三等') || name.includes('third')) {
    return 'badge badge-bronze'
  }
  return 'badge badge-default'
}

const addPrize = async () => {
  if (!canAddPrize.value) return

  try {
    await invoke('add_prize', {
      name: newPrize.value.name,
      description: newPrize.value.description,
      count: newPrize.value.count
    })
    newPrize.value = { name: '', description: '', count: 1 }
    emit('refresh')
  } catch (error) {
    alert('添加失败: ' + error)
  }
}

const removePrize = async (id) => {
  if (!confirm('确定要删除这个奖项吗？')) return

  try {
    await invoke('remove_prize', { id })
    emit('refresh')
  } catch (error) {
    alert('删除失败: ' + error)
  }
}

const openEditModal = (prize) => {
  editingPrize.value = {
    id: prize.id,
    name: prize.name,
    description: prize.description,
    count: prize.total_count,
    usedCount: prize.total_count - prize.remaining_count
  }
  showEditModal.value = true
}

const updatePrize = async () => {
  try {
    await invoke('update_prize', {
      id: editingPrize.value.id,
      name: editingPrize.value.name,
      description: editingPrize.value.description,
      count: editingPrize.value.count
    })
    showEditModal.value = false
    emit('refresh')
  } catch (error) {
    alert('更新失败: ' + error)
  }
}
</script>

<style scoped>
.add-form {
  margin-bottom: 20px;
}

.add-form .form-row {
  align-items: flex-end;
}

.add-form .form-group {
  margin-bottom: 0;
}

.prize-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.prize-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 12px;
  border: 1px solid var(--card-border);
  transition: all 0.3s ease;
}

.prize-item:hover {
  border-color: var(--secondary-color);
  background: rgba(255, 255, 255, 0.05);
}

.prize-item.prize-exhausted {
  opacity: 0.5;
}

.prize-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.prize-name {
  font-weight: 600;
}

.prize-desc {
  color: var(--text-muted);
  font-size: 14px;
}

.prize-count {
  font-size: 13px;
  color: var(--secondary-color);
}

.prize-count strong {
  font-size: 16px;
}

.prize-actions {
  display: flex;
  gap: 8px;
}
</style>
