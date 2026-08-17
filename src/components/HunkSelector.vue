<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { ArrowLeft, ArrowRight, X } from "@lucide/vue";
import { api } from "../lib/bridge";
import { diffLineClass } from "../lib/diff";
import { useEscapeClose } from "../lib/dialog";
import { errorMessage } from "../lib/errors";
import type { DiffHunk, RepositorySnapshot } from "../types";

const props = defineProps<{ repositoryPath: string; file: string }>();
const emit = defineEmits<{ close: []; changed: [snapshot: RepositorySnapshot] }>();
const staged = ref<DiffHunk[]>([]); const unstaged = ref<DiffHunk[]>([]); const busy = ref(false); const error = ref("");
const allHunks = computed(() => [{ label: "Included", staged: true, hunks: staged.value }, { label: "Changes", staged: false, hunks: unstaged.value }]);
const visibleLines = (hunk: DiffHunk) => hunk.patch.split("\n").filter((line) => line.startsWith("@@") || (!line.startsWith("diff ") && !line.startsWith("index ") && !line.startsWith("---") && !line.startsWith("+++")));
async function load() { busy.value = true; error.value = ""; try { [staged.value, unstaged.value] = await Promise.all([api.hunks(props.repositoryPath, props.file, true), api.hunks(props.repositoryPath, props.file, false)]); } catch (caught) { error.value = errorMessage(caught); } finally { busy.value = false; } }
async function apply(hunk: DiffHunk, isStaged: boolean) { busy.value = true; try { const snapshot = await api.applyHunk(props.repositoryPath, props.file, isStaged, hunk.index); emit("changed", snapshot); await load(); } catch (caught) { error.value = errorMessage(caught); busy.value = false; } }
watch(() => props.file, load, { immediate: true });
useEscapeClose(() => emit("close"));
</script>

<template>
  <div class="resolver-backdrop">
    <section class="hunk-selector" role="dialog" aria-modal="true" aria-labelledby="hunk-title">
      <header><div><h2 id="hunk-title">Select Changes</h2><code>{{ file }}</code></div><button class="icon-button" aria-label="Close" @click="emit('close')"><X :size="15" /></button></header>
      <div class="hunk-columns">
        <section v-for="group in allHunks" :key="group.label">
          <div class="hunk-column-title"><strong>{{ group.label }}</strong><span>{{ group.hunks.length }} blocks</span></div>
          <div class="hunk-list">
            <article v-for="hunk in group.hunks" :key="hunk.index" class="hunk-card">
              <header><code>{{ hunk.header }}</code><span class="diff-stat"><b>+{{ hunk.additions }}</b><i>−{{ hunk.deletions }}</i></span><button :title="group.staged ? 'Exclude block from commit' : 'Include block in commit'" :disabled="busy" @click="apply(hunk, group.staged)"><ArrowRight v-if="!group.staged" :size="13" /><ArrowLeft v-else :size="13" />{{ group.staged ? 'Exclude' : 'Include' }}</button></header>
              <pre><code><span v-for="(line, index) in visibleLines(hunk)" :key="index" :class="diffLineClass(line)">{{ line }}
</span></code></pre>
            </article>
            <div v-if="!group.hunks.length" class="hunk-empty">No {{ group.label.toLowerCase() }} blocks.</div>
          </div>
        </section>
      </div>
      <footer><span v-if="error" class="inline-error">{{ error }}</span><span v-else>Each block can move independently between the working tree and index.</span><button @click="emit('close')">Done</button></footer>
    </section>
  </div>
</template>
