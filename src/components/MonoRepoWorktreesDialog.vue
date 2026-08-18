<script setup lang="ts">
import { computed, ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { Check, CircleAlert, FolderGit2, LoaderCircle, X } from "@lucide/vue";
import { api } from "../lib/bridge";
import { useEscapeClose } from "../lib/dialog";
import { errorMessage } from "../lib/errors";
import type { BatchWorktreeResult, WorkspaceSnapshot } from "../types";

const props = defineProps<{ workspace: WorkspaceSnapshot }>();
const emit = defineEmits<{ close: []; complete: [message: string] }>();
const selected = ref(props.workspace.repositories.map((repository) => repository.root));
const targetRoot = ref(`${props.workspace.root}/Worktree`);
const basis = ref<"latestTag" | "defaultBranch">("latestTag");
const busy = ref(false);
const error = ref("");
const result = ref<BatchWorktreeResult>();
const allSelected = computed(() => selected.value.length === props.workspace.repositories.length);

function toggleAll() {
  selected.value = allSelected.value ? [] : props.workspace.repositories.map((repository) => repository.root);
}
function toggleRepository(root: string, checked: boolean) {
  selected.value = checked ? [...selected.value, root] : selected.value.filter((item) => item !== root);
}
async function chooseTarget() {
  const path = await open({ directory: true, multiple: false, title: "Choose Worktree Directory" });
  if (typeof path === "string") targetRoot.value = path;
}
async function submit() {
  busy.value = true; error.value = ""; result.value = undefined;
  try {
    result.value = await api.batchWorktrees(props.workspace.root, selected.value, targetRoot.value, basis.value);
    if (!result.value.failed) emit("complete", `${result.value.succeeded} worktrees created in ${result.value.targetRoot}`);
    else selected.value = result.value.entries.filter((entry) => !entry.success).map((entry) => entry.repositoryPath);
  } catch (caught) { error.value = errorMessage(caught, "Could not create the workspace worktrees."); }
  finally { busy.value = false; }
}
useEscapeClose(() => !busy.value && emit("close"));
</script>

<template>
  <div class="dialog-backdrop" @mousedown.self="!busy && emit('close')">
    <form class="dialog monorepo-dialog" role="dialog" aria-modal="true" aria-labelledby="monorepo-worktrees-title" @submit.prevent="submit">
      <header>
        <div><h2 id="monorepo-worktrees-title">Create Workspace Worktrees</h2><span>{{ workspace.name }} · {{ workspace.repositories.length }} repositories</span></div>
        <button type="button" class="icon-button" aria-label="Close" :disabled="busy" @click="emit('close')"><X :size="15" /></button>
      </header>
      <p>Fetch each selected repository, then create a detached worktree from its newest available tag. Repositories without a tag fall back to their remote default branch.</p>

      <section class="monorepo-repositories" aria-label="Repositories">
        <div class="monorepo-list-heading"><strong>Repositories</strong><button type="button" @click="toggleAll">{{ allSelected ? 'Clear' : 'Select all' }}</button></div>
        <label v-for="repository in workspace.repositories" :key="repository.root" class="monorepo-repository-row">
          <input type="checkbox" :checked="selected.includes(repository.root)" @change="toggleRepository(repository.root, ($event.target as HTMLInputElement).checked)" />
          <FolderGit2 :size="14" />
          <span><strong>{{ repository.name }}</strong><small>{{ repository.root }}</small></span>
          <code>{{ basis === 'latestTag' ? (repository.latestTag ?? repository.defaultBranch ?? 'No base') : (repository.defaultBranch ?? repository.latestTag ?? 'No base') }}</code>
        </label>
      </section>

      <fieldset class="worktree-basis">
        <legend>Starting point</legend>
        <label><input v-model="basis" type="radio" value="latestTag" /><span><strong>Newest tag</strong><small>Fetch tags first; fall back to the remote default branch.</small></span></label>
        <label><input v-model="basis" type="radio" value="defaultBranch" /><span><strong>Remote default branch</strong><small>Prefer origin/master or origin/main; fall back to the newest tag.</small></span></label>
      </fieldset>

      <label class="field-label">Worktree directory</label>
      <div class="path-field"><input v-model="targetRoot" aria-label="Workspace worktree directory" /><button type="button" @click="chooseTarget">Choose…</button></div>
      <p class="path-preview">Creates one folder per repository, for example <code>{{ targetRoot }}/{{ workspace.repositories[0]?.name ?? 'repository' }}</code>.</p>

      <div v-if="error" class="inline-error"><CircleAlert :size="13" />{{ error }}</div>
      <section v-if="result" class="batch-results" aria-live="polite">
        <div v-for="entry in result.entries" :key="entry.repositoryPath" :class="{ failed: !entry.success }">
          <Check v-if="entry.success" :size="13" /><CircleAlert v-else :size="13" />
          <strong>{{ entry.repository }}</strong><span>{{ entry.message }}</span>
        </div>
      </section>
      <footer>
        <span v-if="result" class="batch-summary">{{ result.succeeded }} created · {{ result.failed }} failed</span>
        <button type="button" :disabled="busy" @click="emit('close')">{{ result ? 'Done' : 'Cancel' }}</button>
        <button v-if="!result || result.failed" class="primary-button" :disabled="busy || !selected.length || !targetRoot.trim()" type="submit"><LoaderCircle v-if="busy" :size="13" class="spinning" />{{ busy ? 'Fetching and creating…' : result ? 'Retry Selected' : `Create ${selected.length} Worktrees` }}</button>
      </footer>
    </form>
  </div>
</template>
