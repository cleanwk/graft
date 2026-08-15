<script setup lang="ts">
import { ref } from "vue";
import { ArrowDown, ArrowUp, GripVertical, Play, RotateCcw, X } from "@lucide/vue";
import { api } from "../lib/bridge";
import type { RebaseStep } from "../types";

const props = defineProps<{ repositoryPath: string }>();
const emit = defineEmits<{ close: []; complete: [message: string] }>();
const onto = ref("HEAD~10"); const steps = ref<RebaseStep[]>([]); const busy = ref(false); const error = ref("");
const actions: RebaseStep["action"][] = ["pick", "reword", "edit", "squash", "fixup", "drop"];
async function load() { busy.value = true; error.value = ""; try { steps.value = await api.rebasePlan(props.repositoryPath, onto.value.trim()); if (!steps.value.length) error.value = "No commits exist between the base and HEAD."; } catch (caught) { error.value = String(caught); } finally { busy.value = false; } }
function move(index: number, offset: number) { const target = index + offset; if (target < 0 || target >= steps.value.length) return; const copy = [...steps.value]; [copy[index], copy[target]] = [copy[target], copy[index]]; steps.value = copy; }
async function start() { busy.value = true; error.value = ""; try { const result = await api.startRebase(props.repositoryPath, onto.value.trim(), steps.value); emit("complete", result.summary); } catch (caught) { error.value = String(caught); } finally { busy.value = false; } }
load();
</script>

<template>
  <div class="dialog-backdrop">
    <section class="rebase-dialog" role="dialog" aria-modal="true" aria-labelledby="rebase-title">
      <header><div><h2 id="rebase-title">Interactive Rebase</h2><span>Reorder, combine, edit, or remove commits before replaying them.</span></div><button class="icon-button" aria-label="Close" @click="emit('close')"><X :size="15" /></button></header>
      <div class="rebase-base"><label>Onto</label><input v-model="onto" autofocus spellcheck="false" aria-label="Rebase base" @keyup.enter="load" /><button :disabled="busy" @click="load"><RotateCcw :size="12" />Reload</button></div>
      <div class="rebase-columns"><span>Action</span><span>Commit</span><span>Message</span><span>Order</span></div>
      <div class="rebase-list">
        <div v-for="(step, index) in steps" :key="step.oid" class="rebase-row" :class="`action-${step.action}`">
          <GripVertical :size="13" class="grip" />
          <select v-model="step.action" :aria-label="`Action for ${step.subject}`"><option v-for="action in actions" :key="action" :value="action">{{ action }}</option></select>
          <code>{{ step.shortOid }}</code><span>{{ step.subject }}</span>
          <div><button :disabled="index === 0" aria-label="Move up" @click="move(index, -1)"><ArrowUp :size="12" /></button><button :disabled="index === steps.length - 1" aria-label="Move down" @click="move(index, 1)"><ArrowDown :size="12" /></button></div>
        </div>
        <div v-if="!steps.length && !busy" class="rebase-empty">Choose a valid base to load commits.</div>
      </div>
      <div v-if="error" class="rebase-error">{{ error }}</div>
      <footer><span><strong>Reword</strong> and <strong>Edit</strong> pause so you can amend the commit, then continue.</span><button @click="emit('close')">Cancel</button><button class="primary-button" :disabled="busy || !steps.length" @click="start"><Play :size="12" />{{ busy ? 'Starting…' : 'Start Rebase' }}</button></footer>
    </section>
  </div>
</template>
