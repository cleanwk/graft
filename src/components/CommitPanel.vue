<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import { Check, Copy, FileCode2, Files, GitCommitHorizontal } from "@lucide/vue";
import type { CommitDetail, CommitRow } from "../types";
import { splitCommitPatch, diffLineClass } from "../lib/diff";

const MAX_PATCH_LINES = 1000;

const props = defineProps<{ commit?: CommitRow; detail?: CommitDetail }>();
const selectedFile = ref("");
const filePatches = computed(() => splitCommitPatch(props.detail?.patch ?? ""));
const visiblePatch = computed(() => {
  if (!selectedFile.value) return props.detail?.patch ?? "";
  return filePatches.value.find((file) => file.path === selectedFile.value)?.patch ?? props.detail?.patch ?? "";
});
const allLines = computed(() => visiblePatch.value.split("\n"));
const patchLines = computed(() => allLines.value.slice(0, MAX_PATCH_LINES));
const patchTruncated = computed(() => allLines.value.length > MAX_PATCH_LINES);
const copied = ref(false); let copyTimer: number | undefined;
async function copyHash() {
  if (!props.commit) return;
  await window.navigator.clipboard.writeText(props.commit.oid);
  copied.value = true; window.clearTimeout(copyTimer);
  copyTimer = window.setTimeout(() => { copied.value = false; }, 1500);
}
watch(() => props.commit?.oid, () => { selectedFile.value = ""; });
onBeforeUnmount(() => window.clearTimeout(copyTimer));
</script>

<template>
  <section class="detail-panel" aria-label="Commit details">
    <div v-if="commit" class="detail-content">
      <header class="detail-header">
        <div class="detail-heading">
          <GitCommitHorizontal :size="16" />
          <h2>{{ commit.subject }}</h2>
        </div>
        <button class="icon-button" :class="{ copied }" :title="copied ? 'Commit hash copied' : 'Copy commit hash'" :aria-label="copied ? 'Commit hash copied' : 'Copy commit hash'" @click="copyHash"><Check v-if="copied" :size="14" /><Copy v-else :size="14" /></button>
      </header>
      <div class="commit-meta">
        <span><strong>{{ detail?.author ?? commit.author }}</strong> &lt;{{ detail?.authorEmail ?? commit.authorEmail }}&gt;</span>
        <time>{{ detail?.authoredAt ?? commit.relativeDate }}</time>
        <code>{{ commit.oid }}</code>
      </div>
      <p v-if="detail?.body" class="commit-body">{{ detail.body }}</p>
      <div class="detail-tabs" role="tablist">
        <span class="active" role="tab" aria-selected="true">Changes <span>{{ detail?.files.length ?? 0 }}</span></span>
      </div>
      <div v-if="!detail" class="detail-loading"><span /> <span /> <span /></div>
      <template v-else>
        <div class="changed-files" aria-label="Filter patch by changed file">
          <button class="detail-file all-files" :class="{ selected: !selectedFile }" :aria-pressed="!selectedFile" @click="selectedFile = ''">
            <Files :size="13" />
            <span>All changed files</span>
            <b>{{ detail.files.length }}</b>
          </button>
          <button v-for="file in detail.files" :key="file.path" class="detail-file" :class="{ selected: selectedFile === file.path }" :aria-pressed="selectedFile === file.path" :title="file.path" @click="selectedFile = file.path">
            <span class="file-status" :data-status="file.status[0]">{{ file.status[0] }}</span>
            <FileCode2 :size="13" />
            <span>{{ file.path }}</span>
          </button>
        </div>
        <pre class="patch" aria-label="Commit patch"><code><span v-for="(line, index) in patchLines" :key="index" :class="diffLineClass(line)"><i>{{ index + 1 }}</i>{{ line }}
</span></code></pre>
        <div v-if="patchTruncated" class="patch-truncated">Showing the first {{ MAX_PATCH_LINES.toLocaleString() }} of {{ allLines.length.toLocaleString() }} lines. Select a single file above to narrow the patch.</div>
      </template>
    </div>
    <div v-else class="detail-empty">Select a commit to inspect its changes.</div>
  </section>
</template>
