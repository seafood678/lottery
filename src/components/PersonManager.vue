<template>
  <div class="card">
    <div class="card-header">
      <h2 class="card-title">人员管理</h2>
      <div class="header-actions">
        <button class="btn btn-outline btn-sm" @click="showBatchAdd = true">
          <IconUsersPlus :size="16" />
          批量添加
        </button>
      </div>
    </div>

    <div class="stats">
      <div class="stat-item">
        <div class="stat-value">{{ persons.length }}</div>
        <div class="stat-label">总人数</div>
      </div>
      <div class="stat-item">
        <div class="stat-value">{{ availableCount }}</div>
        <div class="stat-label">未中奖</div>
      </div>
    </div>

    <form @submit.prevent="addPerson" class="add-form">
      <div class="form-row">
        <div class="form-group">
          <input
            v-model="newPerson.name"
            type="text"
            class="form-input"
            placeholder="姓名 *"
            required
          />
        </div>
        <div class="form-group">
          <input
            v-model="newPerson.department"
            type="text"
            class="form-input"
            placeholder="部门 (选填)"
          />
        </div>
        <div class="form-group">
          <input
            v-model="newPerson.employeeId"
            type="text"
            class="form-input"
            placeholder="工号 (选填)"
          />
        </div>
        <button type="submit" class="btn btn-primary" :disabled="!newPerson.name">
          <IconPlus :size="16" />
          添加
        </button>
      </div>
    </form>

    <div class="table-container" v-if="persons.length > 0">
      <table>
        <thead>
          <tr>
            <th>姓名</th>
            <th>部门</th>
            <th>工号</th>
            <th>状态</th>
            <th class="text-right">操作</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="person in persons" :key="person.id" class="fade-in">
            <td>{{ person.name }}</td>
            <td>{{ person.department || '-' }}</td>
            <td>{{ person.employee_id || '-' }}</td>
            <td>
              <span
                :class="['badge', isWinner(person.id) ? 'badge-gold' : 'badge-default']"
              >
                {{ isWinner(person.id) ? '已中奖' : '待抽奖' }}
              </span>
            </td>
            <td class="text-right">
              <button
                class="btn btn-danger btn-sm"
                @click="removePerson(person.id)"
                :disabled="isWinner(person.id)"
              >
                <IconTrash :size="14" />
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="empty-state" v-else>
      <IconUsers class="empty-state-icon" :size="48" />
      <div class="empty-state-text">暂无人员，请添加参与抽奖的人员</div>
    </div>

    <div class="modal-overlay" v-if="showBatchAdd" @click.self="showBatchAdd = false">
      <div class="modal">
        <div class="modal-header">
          <h3 class="modal-title">批量添加人员</h3>
          <button class="modal-close" @click="showBatchAdd = false">&times;</button>
        </div>
        <div class="modal-body">
          <p style="color: var(--text-muted); margin-bottom: 16px; font-size: 14px;">
            每行一个人员，格式：姓名,部门,工号<br />
            部门和工号可省略，例如：<br />
            张三<br />
            李四,技术部<br />
            王五,市场部,EMP001
          </p>
          <textarea
            v-model="batchInput"
            class="form-input batch-textarea"
            rows="10"
            placeholder="请输入人员信息..."
          ></textarea>
        </div>
        <div class="modal-footer">
          <button class="btn btn-outline" @click="showBatchAdd = false">取消</button>
          <button class="btn btn-primary" @click="batchAddPersons">确认添加</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { IconUsers, IconUsersPlus, IconPlus, IconTrash } from '@tabler/icons-vue'

const props = defineProps({
  persons: {
    type: Array,
    default: () => []
  },
  winners: {
    type: Array,
    default: () => []
  }
})

const emit = defineEmits(['refresh'])

const newPerson = ref({
  name: '',
  department: '',
  employeeId: ''
})

const showBatchAdd = ref(false)
const batchInput = ref('')

const availableCount = computed(() => {
  const winnerIds = props.winners.map(w => w.person.id)
  return props.persons.filter(p => !winnerIds.includes(p.id)).length
})

const isWinner = (personId) => {
  return props.winners.some(w => w.person.id === personId)
}

const addPerson = async () => {
  if (!newPerson.value.name) return

  try {
    await invoke('add_person', {
      name: newPerson.value.name,
      department: newPerson.value.department || null,
      employeeId: newPerson.value.employeeId || null
    })
    newPerson.value = { name: '', department: '', employeeId: '' }
    emit('refresh')
  } catch (error) {
    alert('添加失败: ' + error)
  }
}

const removePerson = async (id) => {
  if (!confirm('确定要删除这个人员吗？')) return

  try {
    await invoke('remove_person', { id })
    emit('refresh')
  } catch (error) {
    alert('删除失败: ' + error)
  }
}

const batchAddPersons = async () => {
  const lines = batchInput.value.trim().split('\n').filter(l => l.trim())
  if (lines.length === 0) {
    alert('请输入人员信息')
    return
  }

  const personsData = lines.map(line => {
    const parts = line.split(',').map(p => p.trim())
    return [
      parts[0] || '',
      parts[1] || null,
      parts[2] || null
    ]
  }).filter(p => p[0])

  try {
    await invoke('add_persons_batch', { personsData })
    batchInput.value = ''
    showBatchAdd.value = false
    emit('refresh')
  } catch (error) {
    alert('批量添加失败: ' + error)
  }
}
</script>

<style scoped>
.header-actions {
  display: flex;
  gap: 8px;
}

.add-form {
  margin-bottom: 20px;
}

.add-form .form-row {
  align-items: flex-end;
}

.add-form .form-group {
  margin-bottom: 0;
}

.batch-textarea {
  font-family: monospace;
  resize: vertical;
}

.text-right {
  text-align: right;
}
</style>
