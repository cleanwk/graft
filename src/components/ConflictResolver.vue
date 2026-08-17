<script setup lang="ts">
import { nextTick, ref, watch } from "vue";
import { ArrowDown, Check, X } from "@lucide/vue";
import { api } from "../lib/bridge";
import { useEscapeClose } from "../lib/dialog";
import { errorMessage } from "../lib/errors";
import type { ConflictFile } from "../types";

const props = defineProps<{ repositoryPath: string; file: string }>();
const emit = defineEmits<{ close: []; complete: [message: string] }>();
const conflict = ref<ConflictFile>(); const result = ref(""); const busy = ref(false); const error = ref("");
const resultArea = ref<HTMLTextAreaElement>();
watch(() => props.file, load, { immediate: true });
async function load() { busy.value = true; try { conflict.value = await api.conflict(props.repositoryPath, props.file); result.value = conflict.value.working; } catch (caught) { error.value = errorMessage(caught); } finally { busy.value = false; } }
function use(value: string) { result.value = value; nextTick(() => resultArea.value?.focus()); }
async function save() { busy.value = true; try { const response = await api.resolveConflict(props.repositoryPath, props.file, result.value); emit("complete", response.summary); } catch (caught) { error.value = errorMessage(caught); } finally { busy.value = false; } }
useEscapeClose(() => emit("close"));
</script>

<template>
  <div class="resolver-backdrop">
    <section class="resolver" role="dialog" aria-modal="true" aria-labelledby="resolver-title">
      <header><div><h2 id="resolver-title">Resolve Conflict</h2><code>{{ file }}</code></div><button class="icon-button" aria-label="Close resolver" @click="emit('close')"><X :size="16" /></button></header>
      <div v-if="conflict" class="merge-grid">
        <section><div class="merge-label"><strong>Yours</strong><button @click="use(conflict.ours)">Accept yours <ArrowDown :size="12" /></button></div><textarea readonly :value="conflict.ours" aria-label="Your version" /></section>
        <section><div class="merge-label"><strong>Base</strong><button @click="use(conflict.base)">Use base <ArrowDown :size="12" /></button></div><textarea readonly :value="conflict.base" aria-label="Base version" /></section>
        <section><div class="merge-label"><strong>Theirs</strong><button @click="use(conflict.theirs)">Accept theirs <ArrowDown :size="12" /></button></div><textarea readonly :value="conflict.theirs" aria-label="Their version" /></section>
        <section class="result-pane"><div class="merge-label"><strong>Result</strong><span>Edit the final file, then mark it resolved.</span></div><textarea ref="resultArea" v-model="result" class="merge-result" spellcheck="false" aria-label="Merge result" /></section>
      </div>
      <div v-else class="resolver-loading">{{ error || 'Loading conflict versions…' }}</div>
      <footer><span v-if="error" class="inline-error">{{ error }}</span><button @click="emit('close')">Cancel</button><button class="primary-button" :disabled="busy || !conflict" @click="save"><Check :size="13" />Mark Resolved</button></footer>
    </section>
  </div>
</template>

