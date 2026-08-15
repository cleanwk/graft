<script setup lang="ts">
import { onMounted, ref } from "vue";
import { Download, RefreshCw, X } from "@lucide/vue";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

const version = ref("");
const notes = ref("");
const visible = ref(false);
const installing = ref(false);
const progress = ref<number>();
const error = ref("");
let availableUpdate: Awaited<ReturnType<typeof check>>;

async function checkForUpdate() {
  if (import.meta.env.DEV) return;
  try {
    availableUpdate = await check({ timeout: 15_000 });
    if (!availableUpdate) return;
    version.value = availableUpdate.version;
    notes.value = availableUpdate.body ?? "A new Graft beta is ready.";
    visible.value = true;
  } catch (caught) {
    console.warn("Graft update check failed", caught);
  }
}

async function installUpdate() {
  if (!availableUpdate || installing.value) return;
  installing.value = true;
  error.value = "";
  let downloaded = 0;
  let total: number | undefined;
  try {
    await availableUpdate.downloadAndInstall((event) => {
      if (event.event === "Started") total = event.data.contentLength;
      if (event.event === "Progress") downloaded += event.data.chunkLength;
      progress.value = total ? Math.min(100, Math.round((downloaded / total) * 100)) : undefined;
    });
    await relaunch();
  } catch (caught) {
    error.value = `Update failed: ${String(caught)}`;
    installing.value = false;
  }
}

onMounted(checkForUpdate);
</script>

<template>
  <aside v-if="visible" class="update-banner" aria-live="polite">
    <Download :size="17" />
    <div>
      <strong>Graft {{ version }} is ready</strong>
      <span>{{ error || notes }}</span>
    </div>
    <button class="primary-button" :disabled="installing" @click="installUpdate">
      <RefreshCw :size="13" :class="{ spinning: installing }" />
      {{ installing ? (progress === undefined ? 'Updating…' : `Updating ${progress}%`) : 'Update & Restart' }}
    </button>
    <button class="icon-button" aria-label="Dismiss update" :disabled="installing" @click="visible = false"><X :size="14" /></button>
  </aside>
</template>
