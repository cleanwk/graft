<script setup lang="ts">
import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { X } from "@lucide/vue";
import { api } from "../lib/bridge";
import type { RepositorySnapshot } from "../types";

const props = defineProps<{ repository: RepositorySnapshot }>();
const emit = defineEmits<{ close: []; complete: [message: string] }>();
const selectedBranch = ref(props.repository.branch);
const path = ref(`${props.repository.root}-worktrees/${props.repository.branch}`);
const createBranch = ref(false); const branchName = ref(""); const busy = ref(false); const error = ref("");
const targetBranch = computed(() => createBranch.value ? branchName.value.trim() : selectedBranch.value);

async function choosePath() {
  const chosen = await open({ directory: true, multiple: false, title: "Choose Worktree Parent Folder" });
  if (typeof chosen === "string") path.value = `${chosen}/${targetBranch.value.replaceAll('/', '-')}`;
}
async function submit() {
  busy.value = true; error.value = "";
  try { const result = await api.addWorktree(props.repository.root, path.value, targetBranch.value, createBranch.value); emit("complete", result.summary); }
  catch (caught) { error.value = typeof caught === "string" ? caught : (caught as { message?: string }).message ?? "Could not create the worktree."; }
  finally { busy.value = false; }
}
</script>

<template>
  <div class="dialog-backdrop" @mousedown.self="emit('close')">
    <form class="dialog" role="dialog" aria-modal="true" aria-labelledby="worktree-title" @submit.prevent="submit">
      <header><h2 id="worktree-title">New Worktree</h2><button type="button" class="icon-button" aria-label="Close" @click="emit('close')"><X :size="15" /></button></header>
      <p>Check out a branch into a separate working directory. Your current work stays untouched.</p>
      <label class="field-label">Branch</label>
      <select v-model="selectedBranch" :disabled="createBranch">
        <option v-for="branch in repository.branches.filter(b => !b.remote)" :key="branch.name" :value="branch.name">{{ branch.name }}</option>
      </select>
      <label class="checkbox-row"><input v-model="createBranch" type="checkbox" />Create a new branch</label>
      <input v-if="createBranch" v-model="branchName" placeholder="feature/my-work" aria-label="New branch name" />
      <label class="field-label">Worktree location</label>
      <div class="path-field"><input v-model="path" aria-label="Worktree location" /><button type="button" @click="choosePath">Choose…</button></div>
      <div v-if="error" class="inline-error">{{ error }}</div>
      <footer><button type="button" @click="emit('close')">Cancel</button><button class="primary-button" :disabled="busy || !path.trim() || !targetBranch" type="submit">{{ busy ? 'Creating…' : 'Create Worktree' }}</button></footer>
    </form>
  </div>
</template>
