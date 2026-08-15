<script setup lang="ts">
import { computed, ref } from "vue";
import { Check, ChevronDown, File, GitCommit, RotateCcw } from "@lucide/vue";
import type { Change } from "../types";

const props = defineProps<{ changes: Change[]; busy?: boolean; truncated?: boolean }>();
const emit = defineEmits<{ staged: [path: string, staged: boolean]; commit: [message: string, amend: boolean, pushAfter: boolean]; resolve: [path: string]; inspect: [path: string] }>();
const message = ref(""); const amend = ref(false);
const showCommitMenu = ref(false);
const staged = computed(() => props.changes.filter((change) => change.staged && !change.conflicted));
const unstaged = computed(() => props.changes.filter((change) => !change.staged || change.conflicted));

function submit(pushAfter = false) {
  if (message.value.trim() && staged.value.length) emit("commit", message.value, amend.value, pushAfter);
  showCommitMenu.value = false;
}
defineExpose({ clear: () => { message.value = ""; amend.value = false; }, submit });
</script>

<template>
  <aside class="commit-tool" aria-label="Commit tool window">
    <header class="panel-title"><GitCommit :size="14" /><span>Commit</span><b>{{ changes.length }}</b></header>
    <div class="change-groups">
      <div v-if="truncated" class="change-limit">Showing the first 2,000 changes. Use a path filter or the command line for this unusually large change set.</div>
      <section v-if="staged.length">
        <h3>Included in commit <span>{{ staged.length }}</span></h3>
        <label v-for="change in staged" :key="change.path" class="change-row" @dblclick="emit('inspect', change.path)">
          <input type="checkbox" checked @change="emit('staged', change.path, false)" />
          <File :size="13" /><span :title="change.path">{{ change.path }}</span><i :data-status="change.indexStatus">{{ change.indexStatus }}</i>
        </label>
      </section>
      <section v-if="unstaged.length">
        <h3>Changes <span>{{ unstaged.length }}</span></h3>
        <label v-for="change in unstaged" :key="change.path" class="change-row" :class="{ conflict: change.conflicted }" @dblclick="change.conflicted ? emit('resolve', change.path) : emit('inspect', change.path)">
          <input type="checkbox" @change="emit('staged', change.path, true)" />
          <File :size="13" /><span :title="change.path">{{ change.path }}</span><button v-if="change.conflicted" type="button" class="resolve-link" @click.prevent="emit('resolve', change.path)">Resolve…</button><i v-else :data-status="change.worktreeStatus">{{ change.worktreeStatus }}</i>
        </label>
      </section>
      <div v-if="!changes.length" class="clean-state"><Check :size="18" /><strong>No changes</strong><span>Working tree is clean.</span></div>
    </div>
    <form class="commit-form" @submit.prevent="submit(false)">
      <textarea v-model="message" placeholder="Commit message" aria-label="Commit message" />
      <label class="amend"><input v-model="amend" type="checkbox" /><RotateCcw :size="12" />Amend</label>
      <div class="commit-actions">
        <button class="primary-button" type="submit" :disabled="!message.trim() || !staged.length || busy">Commit</button>
        <button class="primary-menu" type="button" aria-label="More commit options" :disabled="!message.trim() || !staged.length || busy" @click="showCommitMenu = !showCommitMenu"><ChevronDown :size="13" /></button>
        <div v-if="showCommitMenu" class="commit-menu"><button type="button" :disabled="busy" @click="submit(false)">Commit <kbd>⌘↩</kbd></button><button type="button" :disabled="busy" @click="submit(true)">Commit and Push…</button></div>
      </div>
    </form>
  </aside>
</template>
