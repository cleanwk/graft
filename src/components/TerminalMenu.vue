<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { ChevronDown, SquareTerminal } from "@lucide/vue";
import { api } from "../lib/bridge";
import { errorMessage } from "../lib/errors";
import type { TerminalApp } from "../types";

const props = defineProps<{ path: string }>();
const emit = defineEmits<{ error: [message: string] }>();
const root = ref<HTMLElement>();
const terminals = ref<TerminalApp[]>([]);
const open = ref(false);
const busy = ref(false);
const preferred = computed(() => terminals.value.find((terminal) => terminal.id === localStorage.getItem("graft.preferredTerminal")) ?? terminals.value[0]);

async function launch(terminal = preferred.value) {
  if (!terminal || busy.value) return;
  busy.value = true; open.value = false;
  try { await api.openTerminal(props.path, terminal.id); localStorage.setItem("graft.preferredTerminal", terminal.id); }
  catch (caught) { emit("error", errorMessage(caught, `Could not open ${terminal.name}.`)); }
  finally { busy.value = false; }
}
function closeOutside(event: PointerEvent) { if (!root.value?.contains(event.target as Node)) open.value = false; }
onMounted(async () => {
  document.addEventListener("pointerdown", closeOutside);
  try { terminals.value = await api.terminals(); } catch { terminals.value = []; }
});
onBeforeUnmount(() => document.removeEventListener("pointerdown", closeOutside));
</script>

<template>
  <div ref="root" class="terminal-control">
    <button class="terminal-main" :disabled="!preferred || busy" :title="preferred ? `Open ${path} in ${preferred.name}` : 'No supported terminal found'" @click="launch()"><SquareTerminal :size="14" /><span>{{ preferred?.name ?? 'Terminal' }}</span></button>
    <button class="terminal-chevron" :disabled="!terminals.length || busy" aria-label="Choose terminal" :aria-expanded="open" @click.stop="open = !open"><ChevronDown :size="11" /></button>
    <div v-if="open" class="terminal-menu" role="menu">
      <span>Open repository in</span>
      <button v-for="terminal in terminals" :key="terminal.id" role="menuitem" @click="launch(terminal)"><SquareTerminal :size="13" />{{ terminal.name }}<b v-if="terminal.id === preferred?.id">Default</b></button>
    </div>
  </div>
</template>
