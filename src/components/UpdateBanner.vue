<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { CheckCircle2, Download, RefreshCw, X } from "@lucide/vue";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";

const props = withDefaults(defineProps<{ checkRequest?: number }>(), { checkRequest: 0 });
const version = ref("");
const notes = ref("");
const visible = ref(false);
const installing = ref(false);
const progress = ref<number>();
const error = ref("");
const upToDate = ref(false);
let availableUpdate: Awaited<ReturnType<typeof check>>;
let timer: number | undefined;
let hideTimer: number | undefined;

async function checkForUpdate(showResult = false) {
  if (import.meta.env.DEV) return;
  window.clearTimeout(hideTimer);
  upToDate.value = false;
  try {
    availableUpdate = await check({ timeout: 15_000 });
    if (!availableUpdate) {
      visible.value = false;
      if (showResult) {
        upToDate.value = true;
        visible.value = true;
        hideTimer = window.setTimeout(() => { visible.value = false; }, 4_000);
      }
      return;
    }
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

function checkWhenVisible() {
  if (document.visibilityState === "visible") void checkForUpdate();
}

watch(() => props.checkRequest, () => { void checkForUpdate(true); });
onMounted(() => {
  void checkForUpdate();
  timer = window.setInterval(() => { void checkForUpdate(); }, 4 * 60 * 60 * 1_000);
  window.addEventListener("online", checkWhenVisible);
  document.addEventListener("visibilitychange", checkWhenVisible);
});
onBeforeUnmount(() => {
  window.clearInterval(timer);
  window.clearTimeout(hideTimer);
  window.removeEventListener("online", checkWhenVisible);
  document.removeEventListener("visibilitychange", checkWhenVisible);
});
</script>

<template>
  <aside v-if="visible" class="update-banner" aria-live="polite">
    <CheckCircle2 v-if="upToDate" :size="17" />
    <Download v-else :size="17" />
    <div>
      <strong>{{ upToDate ? 'Graft is up to date' : `Graft ${version} is ready` }}</strong>
      <span>{{ upToDate ? 'You already have the latest version.' : (error || notes) }}</span>
    </div>
    <button v-if="!upToDate" class="primary-button" :disabled="installing" @click="installUpdate">
      <RefreshCw :size="13" :class="{ spinning: installing }" />
      {{ installing ? (progress === undefined ? 'Updating…' : `Updating ${progress}%`) : 'Update & Restart' }}
    </button>
    <button class="icon-button" aria-label="Dismiss update" :disabled="installing" @click="visible = false"><X :size="14" /></button>
  </aside>
</template>
