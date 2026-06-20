<template>
  <div class="user-panel">
    <div class="panel-header">
      <div class="header-left">
        <span class="icon-user">[!]</span>
        <h2 class="panel-title">USER MANAGEMENT</h2>
      </div>
    </div>

    <div class="section">
      <h3 class="section-title">[USERS] REGISTERED USERS</h3>
      <p class="text-muted" style="margin-bottom:8px;font-size:10px;">PER-FILE USERNAME + PASSWORD AUTH WITH ARGON2 HASHING. ROLE-BASED ACCESS: ADMIN, USER, VIEWER.</p>
      <div v-if="store.users.length === 0" class="text-muted" style="font-size:10px;">NO USERS REGISTERED</div>
      <div class="user-list">
        <div v-for="user in store.users" :key="user.id" class="user-card">
          <div class="user-header">
            <span class="user-name">{{ user.username }}</span>
            <span class="user-role">{{ user.role }}</span>
            <span class="user-active" :class="{ on: user.isActive }">{{ user.isActive ? 'ACTIVE' : 'INACTIVE' }}</span>
            <div class="user-actions">
              <button class="user-action-btn" @click="handleRole(user.id, user.role === 'admin' ? 'user' : 'admin')">[ROLE]</button>
              <button class="user-action-btn" @click="handleDelete(user.id)">[DEL]</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <div class="create-section" v-if="showCreate">
      <h3 class="section-title">[+] CREATE USER</h3>
      <input v-model="newUsername" class="bw-input" placeholder="USERNAME" @keyup.enter="handleCreate" />
      <input v-model="newPassword" class="bw-input" type="password" placeholder="PASSWORD" @keyup.enter="handleCreate" />
      <select v-model="newRole" class="bw-input" style="appearance:none;">
        <option value="user">USER</option>
        <option value="admin">ADMIN</option>
        <option value="viewer">VIEWER</option>
      </select>
      <button class="bw-btn" @click="handleCreate">[CREATE]</button>
    </div>
    <button v-else class="bw-btn" @click="showCreate = true">[+ ADD USER]</button>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'

const store = useAppStore()
const newUsername = ref('')
const newPassword = ref('')
const newRole = ref('user')
const showCreate = ref(false)

onMounted(() => {
  store.fetchUsers()
})

async function handleCreate() {
  if (!newUsername.value.trim() || !newPassword.value.trim()) return
  await store.createUser(newUsername.value.trim(), newPassword.value.trim(), newRole.value)
  newUsername.value = ''
  newPassword.value = ''
  newRole.value = 'user'
  showCreate.value = false
}

async function handleDelete(userId: string) {
  await store.deleteUser(userId)
}

async function handleRole(userId: string, role: string) {
  await store.updateUserRole(userId, role)
}
</script>

<style scoped>
.user-panel {
  width: 100%;
  height: 100%;
  overflow-y: auto;
  padding: 20px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  background: transparent;
}

.panel-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--border-glass);
  margin-bottom: 16px;
}

.header-left { display: flex; align-items: center; gap: 8px; }
.icon-user { font-size: 14px; color: var(--text-accent); }
.panel-title { font-size: 13px; font-weight: 700; letter-spacing: 1px; margin: 0; color: var(--text-primary); }

.section { margin-bottom: 16px; }

.section-title {
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  color: var(--text-secondary);
  margin: 0 0 8px;
  font-family: var(--font-mono);
}

.user-list { display: flex; flex-direction: column; gap: 8px; }

.user-card {
  border: 1px solid var(--border-glass);
  padding: 10px 12px;
  background: var(--bg-glass-light);
  backdrop-filter: blur(var(--glass-blur-light));
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
}

.user-card:hover {
  border-color: var(--border-accent);
  background: var(--accent-dim);
}

.user-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.user-name { font-size: 12px; font-weight: 600; flex: 1; color: var(--text-primary); }
.user-role { font-size: 9px; border: 1px solid var(--border-medium); padding: 1px 6px; color: var(--text-muted); border-radius: var(--radius-sm); }
.user-active { font-size: 9px; font-weight: 600; }
.user-active.on { color: var(--text-accent); }
.user-active:not(.on) { color: var(--text-muted); opacity: 0.3; }

.user-actions {
  display: flex;
  gap: 4px;
}

.user-action-btn {
  background: transparent;
  border: 1px solid var(--border-medium);
  color: var(--text-muted);
  padding: 2px 6px;
  font-family: var(--font-mono);
  font-size: 8px;
  font-weight: 600;
  cursor: pointer;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.user-action-btn:hover {
  background: var(--accent-dim);
  border-color: var(--border-accent);
  color: var(--text-accent);
}

.create-section {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid var(--border-glass);
}

.bw-input {
  background: var(--bg-surface);
  border: 1px solid var(--border-medium);
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: 11px;
  padding: 6px 8px;
  width: 100%;
  margin-bottom: 6px;
  border-radius: var(--radius-sm);
}

.bw-input::placeholder {
  color: var(--text-muted);
  opacity: 0.5;
}

.bw-input:focus {
  border-color: var(--border-accent);
  outline: none;
}

.bw-btn {
  background: var(--accent);
  color: var(--text-inverse);
  border: 1px solid var(--accent);
  padding: 6px 14px;
  cursor: pointer;
  font-family: var(--font-mono);
  font-size: 11px;
  font-weight: 600;
  width: 100%;
  border-radius: var(--radius-sm);
  transition: all var(--transition-fast);
}

.bw-btn:hover {
  background: #00cc35;
  border-color: #00cc35;
}

.text-muted { color: var(--text-muted) !important; }
</style>
