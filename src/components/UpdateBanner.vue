<script setup lang="ts">
import { onMounted, ref } from "vue";
import { Check, CircleAlert, Download, LoaderCircle, RefreshCw, X } from "@lucide/vue";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

const version = ref("");
const notes = ref("");
const visible = ref(false);
const checking = ref(false);
const installing = ref(false);
const progress = ref<number>();
const error = ref("");
const upToDate = ref(false);
let availableUpdate: Awaited<ReturnType<typeof check>>;

async function checkForUpdate(manual = false) {
  if (checking.value || installing.value) return;
  checking.value = true;
  error.value = "";
  upToDate.value = false;
  if (manual) visible.value = true;
  try {
    availableUpdate = await check({ timeout: 15_000 });
    if (availableUpdate) {
      version.value = availableUpdate.version;
      notes.value = availableUpdate.body ?? "A new Graft beta is ready.";
      visible.value = true;
    } else if (manual) {
      upToDate.value = true;
    }
  } catch (caught) {
    if (manual) error.value = `Couldn't check for updates: ${String(caught)}`;
    else console.warn("Graft update check failed", caught);
  } finally {
    checking.value = false;
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

defineExpose({ checkForUpdate });
onMounted(() => checkForUpdate());
</script>

<template>
  <aside v-if="visible" class="update-banner" aria-live="polite">
    <LoaderCircle v-if="checking" :size="17" class="spinning" />
    <CircleAlert v-else-if="error" :size="17" />
    <Check v-else-if="upToDate" :size="17" />
    <Download v-else :size="17" />
    <div>
      <strong v-if="checking">Checking for updates…</strong>
      <strong v-else-if="error">Update problem</strong>
      <strong v-else-if="upToDate">Graft is up to date</strong>
      <strong v-else>Graft {{ version }} is ready</strong>
      <span v-if="error || (!checking && !upToDate)">{{ error || notes }}</span>
    </div>
    <button v-if="availableUpdate && !upToDate" class="primary-button" :disabled="installing" @click="installUpdate">
      <RefreshCw :size="13" :class="{ spinning: installing }" />
      {{ installing ? (progress === undefined ? 'Updating…' : `Updating ${progress}%`) : 'Update & Restart' }}
    </button>
    <button class="icon-button" aria-label="Dismiss update" :disabled="installing || checking" @click="visible = false"><X :size="14" /></button>
  </aside>
</template>
