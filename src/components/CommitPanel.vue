<script setup lang="ts">
import { computed } from "vue";
import { Copy, FileCode2, GitCommitHorizontal } from "@lucide/vue";
import type { CommitDetail, CommitRow } from "../types";

const props = defineProps<{ commit?: CommitRow; detail?: CommitDetail }>();
const emit = defineEmits<{ close: [] }>();
const patchLines = computed(() => props.detail?.patch.split("\n").slice(0, 2000) ?? []);
const lineClass = (line: string) => line.startsWith("+") && !line.startsWith("+++") ? "addition" : line.startsWith("-") && !line.startsWith("---") ? "deletion" : line.startsWith("@@") ? "hunk" : "";
const copyHash = () => props.commit && window.navigator.clipboard.writeText(props.commit.oid);
</script>

<template>
  <section class="detail-panel" aria-label="Commit details">
    <div v-if="commit" class="detail-content">
      <header class="detail-header">
        <div class="detail-heading">
          <GitCommitHorizontal :size="16" />
          <h2>{{ commit.subject }}</h2>
        </div>
        <button class="icon-button" title="Copy commit hash" aria-label="Copy commit hash" @click="copyHash"><Copy :size="14" /></button>
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
        <div class="changed-files" aria-label="Changed files">
          <div v-for="file in detail.files" :key="file.path" class="detail-file">
            <span class="file-status" :data-status="file.status[0]">{{ file.status[0] }}</span>
            <FileCode2 :size="13" />
            <span>{{ file.path }}</span>
          </div>
        </div>
        <pre class="patch" aria-label="Commit patch"><code><span v-for="(line, index) in patchLines" :key="index" :class="lineClass(line)"><i>{{ index + 1 }}</i>{{ line }}
</span></code></pre>
      </template>
    </div>
    <div v-else class="detail-empty">Select a commit to inspect its changes.</div>
  </section>
</template>
