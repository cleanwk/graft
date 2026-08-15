<script setup lang="ts">
import { computed, ref } from "vue";
import { useVirtualizer } from "@tanstack/vue-virtual";
import { SearchX, Tag } from "@lucide/vue";
import type { CommitRow } from "../types";
import { allocateGraphRows } from "../lib/graph";

const ROW_HEIGHT = 31;

const props = defineProps<{ commits: CommitRow[]; selected?: string; hasMore: boolean; loadingMore: boolean; filtered?: boolean }>();
const emit = defineEmits<{ select: [commit: CommitRow]; more: [] }>();
const scroller = ref<HTMLElement>();
const virtualizer = useVirtualizer(computed(() => ({ count: props.commits.length, getScrollElement: () => scroller.value ?? null, estimateSize: () => ROW_HEIGHT, overscan: 16 })));
const colors = Array.from({ length: 8 }, (_, index) => `var(--graph-${index + 1})`);
const graphRows = computed(() => allocateGraphRows(props.commits, colors.length));
const branchTransitions = (index: number) => graphRows.value[index].parentLanes.filter((lane) => lane !== graphRows.value[index].lane);

function onScroll() {
  const element = scroller.value;
  if (element && props.hasMore && !props.loadingMore && element.scrollTop + element.clientHeight > element.scrollHeight - 900) emit("more");
}

function onKeydown(event: KeyboardEvent) {
  const count = props.commits.length;
  if (!count) return;
  const current = props.commits.findIndex((commit) => commit.oid === props.selected);
  const pageSize = Math.max(1, Math.floor((scroller.value?.clientHeight ?? ROW_HEIGHT * 10) / ROW_HEIGHT) - 1);
  let next: number;
  if (event.key === "ArrowDown") next = current < 0 ? 0 : Math.min(current + 1, count - 1);
  else if (event.key === "ArrowUp") next = current < 0 ? 0 : Math.max(current - 1, 0);
  else if (event.key === "PageDown") next = current < 0 ? 0 : Math.min(current + pageSize, count - 1);
  else if (event.key === "PageUp") next = current < 0 ? 0 : Math.max(current - pageSize, 0);
  else if (event.key === "Home") next = 0;
  else if (event.key === "End") next = count - 1;
  else return;
  if (next === current) { event.preventDefault(); return; }
  event.preventDefault();
  emit("select", props.commits[next]);
  virtualizer.value.scrollToIndex(next, { align: "auto" });
}
</script>

<template>
  <div ref="scroller" class="commit-scroll" role="region" aria-label="Commit log" tabindex="0" @scroll="onScroll" @keydown="onKeydown">
    <div class="commit-spacer" :style="{ height: `${virtualizer.getTotalSize()}px` }">
      <button
        v-for="row in virtualizer.getVirtualItems()"
        :key="commits[row.index].oid"
        class="commit-row"
        :class="{ selected: commits[row.index].oid === selected, odd: row.index % 2 === 1 }"
        :style="{ transform: `translateY(${row.start}px)` }"
        :aria-label="`${commits[row.index].subject}, ${commits[row.index].author}, ${commits[row.index].relativeDate}`"
        tabindex="-1"
        @click="emit('select', commits[row.index])"
      >
        <span class="graph-cell" aria-hidden="true">
          <svg viewBox="0 0 72 31" preserveAspectRatio="none">
            <line v-for="lane in graphRows[row.index].before" :key="`before-${lane}`" :x1="8 + (lane - 1) * 8" y1="0" :x2="8 + (lane - 1) * 8" y2="15.5" :stroke="colors[(lane - 1) % colors.length]" />
            <line v-for="lane in graphRows[row.index].nextLanes" :key="`after-${lane}`" :x1="8 + lane * 8" y1="15.5" :x2="8 + lane * 8" y2="31" :stroke="colors[lane % colors.length]" />
            <line v-for="parentLane in branchTransitions(row.index)" :key="`parent-${parentLane}`" :x1="8 + graphRows[row.index].lane * 8" y1="15.5" :x2="8 + parentLane * 8" y2="31" :stroke="colors[parentLane % colors.length]" />
            <circle :cx="8 + graphRows[row.index].lane * 8" cy="15.5" r="4" fill="var(--commit-row-fill)" :stroke="colors[graphRows[row.index].lane % colors.length]" stroke-width="2" />
          </svg>
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
    <div v-else-if="!commits.length" class="commit-empty">
      <template v-if="filtered">
        <SearchX :size="20" />
        <strong>No matching commits</strong>
        <span>No loaded commit matches the current search.</span>
      </template>
      <template v-else>
        <strong>No history yet</strong>
        <span>Commits will appear here once this repository has history.</span>
      </template>
    </div>
  </div>
</template>
