<script setup lang="ts">
import { computed, ref } from "vue";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { GitCommit, Tag } from "@lucide/vue";
import type { CommitRow } from "../types";

const props = defineProps<{ commits: CommitRow[]; selected?: string; hasMore: boolean; loadingMore: boolean }>();
const emit = defineEmits<{ select: [commit: CommitRow]; more: [] }>();
const scroller = ref<HTMLElement>();
const virtualizer = useVirtualizer(computed(() => ({ count: props.commits.length, getScrollElement: () => scroller.value ?? null, estimateSize: () => 31, overscan: 16 })));
const lanes = ["#4d72e8", "#d17442", "#4d9b76", "#9b6bc1", "#c0973d"];

function onScroll() {
  const element = scroller.value;
  if (element && props.hasMore && !props.loadingMore && element.scrollTop + element.clientHeight > element.scrollHeight - 900) emit("more");
}
</script>

<template>
  <div ref="scroller" class="commit-scroll" role="region" aria-label="Commit log" tabindex="0" @scroll="onScroll">
    <div class="commit-spacer" :style="{ height: `${virtualizer.getTotalSize()}px` }">
      <button
        v-for="row in virtualizer.getVirtualItems()"
        :key="commits[row.index].oid"
        class="commit-row"
        :class="{ selected: commits[row.index].oid === selected }"
        :style="{ transform: `translateY(${row.start}px)` }"
        :aria-label="`${commits[row.index].subject}, ${commits[row.index].author}, ${commits[row.index].relativeDate}`"
        @click="emit('select', commits[row.index])"
      >
        <span class="graph-cell" aria-hidden="true">
          <span class="graph-line" :style="{ backgroundColor: lanes[row.index % lanes.length] }" />
          <GitCommit :size="13" :stroke-width="2.5" :style="{ color: lanes[row.index % lanes.length] }" />
        </span>
        <span class="commit-subject">{{ commits[row.index].subject }}</span>
        <span v-if="commits[row.index].decorations.length" class="decorations">
          <span v-for="decoration in commits[row.index].decorations.slice(0, 2)" :key="decoration" class="ref-label"><Tag :size="10" />{{ decoration.replace('HEAD -> ', '') }}</span>
        </span>
        <span class="commit-author">{{ commits[row.index].author }}</span>
        <span class="commit-hash">{{ commits[row.index].shortOid }}</span>
        <span class="commit-date">{{ commits[row.index].relativeDate }}</span>
      </button>
    </div>
    <div v-if="loadingMore" class="loading-more">Loading more history…</div>
  </div>
</template>
