<script setup lang="ts">
import { computed, ref } from "vue";
import { CircleAlert, X } from "@lucide/vue";
import { api } from "../lib/bridge";

const props = defineProps<{ repositoryPath: string; operation: "merge" | "cherryPick" | "revert" | "reset"; initialTarget?: string }>();
const emit = defineEmits<{ close: []; complete: [message: string] }>();
const target = ref(props.initialTarget ?? ""); const mode = ref("mixed"); const busy = ref(false); const error = ref("");
const label = computed(() => ({ merge: "Merge", cherryPick: "Cherry-pick", revert: "Revert", reset: "Reset" })[props.operation]);
const dangerous = computed(() => props.operation === "reset" && mode.value === "hard");
async function submit() {
  busy.value = true; error.value = "";
  try { const result = await api.historyOperation(props.repositoryPath, props.operation, target.value.trim(), props.operation === "reset" ? mode.value : undefined); emit("complete", result.summary); }
  catch (caught) { error.value = typeof caught === "string" ? caught : (caught as { message?: string }).message ?? `${label.value} failed.`; }
  finally { busy.value = false; }
}
</script>

<template>
  <div class="dialog-backdrop" @mousedown.self="emit('close')">
    <form class="dialog operation-dialog" role="dialog" aria-modal="true" :aria-labelledby="`operation-${operation}`" @submit.prevent="submit">
      <header><h2 :id="`operation-${operation}`">{{ label }}</h2><button type="button" class="icon-button" aria-label="Close" @click="emit('close')"><X :size="15" /></button></header>
      <p v-if="operation === 'merge'">Merge the named branch into the current branch using Git's configured merge strategy.</p>
      <p v-else-if="operation === 'cherryPick'">Apply the selected commit on top of the current branch.</p>
      <p v-else-if="operation === 'revert'">Create a new commit that reverses the selected commit.</p>
      <p v-else>Move the current branch and choose what happens to the index and working tree.</p>
      <label class="field-label">{{ operation === 'merge' ? 'Branch or revision' : 'Commit' }}</label>
      <input v-model="target" autofocus spellcheck="false" :placeholder="operation === 'merge' ? 'feature/my-branch' : 'Commit hash'" />
      <template v-if="operation === 'reset'">
        <label class="field-label">Reset mode</label>
        <div class="reset-options">
          <label><input v-model="mode" type="radio" value="soft" /><span><strong>Soft</strong>Keep index and working tree</span></label>
          <label><input v-model="mode" type="radio" value="mixed" /><span><strong>Mixed</strong>Reset index, keep working tree</span></label>
          <label><input v-model="mode" type="radio" value="hard" /><span><strong>Hard</strong>Discard index and working tree changes</span></label>
        </div>
        <div v-if="dangerous" class="danger-warning"><CircleAlert :size="15" /><span>Hard reset permanently discards uncommitted changes. This cannot be undone by Graft.</span></div>
      </template>
      <div v-if="error" class="inline-error">{{ error }}</div>
      <footer><button type="button" @click="emit('close')">Cancel</button><button class="primary-button" :class="{ destructive: dangerous }" :disabled="busy || !target.trim()" type="submit">{{ busy ? `${label}ing…` : dangerous ? 'Hard Reset' : label }}</button></footer>
    </form>
  </div>
</template>

