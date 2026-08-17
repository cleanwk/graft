<script setup lang="ts">
import { computed, ref } from "vue";
import { X } from "@lucide/vue";
import { api } from "../lib/bridge";
import { useEscapeClose } from "../lib/dialog";
import { errorMessage } from "../lib/errors";

const props = defineProps<{ repositoryPath: string; kind: "branch" | "tag" | "remote" }>();
const emit = defineEmits<{ close: []; complete: [message: string] }>();
const name = ref(""); const value = ref(props.kind === "remote" ? "" : "HEAD"); const busy = ref(false); const error = ref("");
const title = computed(() => ({ branch: "New Branch", tag: "New Tag", remote: "Add Remote" })[props.kind]);
const valueLabel = computed(() => props.kind === "remote" ? "Remote URL" : props.kind === "branch" ? "Start point" : "Target commit");
async function submit() {
  busy.value = true;
  try {
    const action = props.kind === "remote" ? "add" : "create";
    const result = await api.manageReference(props.repositoryPath, props.kind, action, name.value.trim(), value.value.trim());
    emit("complete", result.summary);
  } catch (caught) { error.value = errorMessage(caught); }
  finally { busy.value = false; }
}
useEscapeClose(() => emit("close"));
</script>

<template>
  <div class="dialog-backdrop" @mousedown.self="emit('close')">
    <form class="dialog" role="dialog" aria-modal="true" aria-labelledby="reference-title" @submit.prevent="submit">
      <header><h2 id="reference-title">{{ title }}</h2><button type="button" class="icon-button" aria-label="Close" @click="emit('close')"><X :size="15" /></button></header>
      <p v-if="kind === 'branch'">Create and switch to a branch without changing your uncommitted work.</p>
      <p v-else-if="kind === 'tag'">Create a lightweight tag at a commit or revision.</p>
      <p v-else>Add a named remote. Credentials remain managed by system Git.</p>
      <label class="field-label">Name</label>
      <input v-model="name" autofocus spellcheck="false" :placeholder="kind === 'remote' ? 'origin' : kind === 'branch' ? 'feature/my-work' : 'v1.0.0'" />
      <label class="field-label">{{ valueLabel }}</label>
      <input v-model="value" spellcheck="false" :placeholder="kind === 'remote' ? 'git@github.com:owner/repository.git' : 'HEAD'" />
      <div v-if="error" class="inline-error">{{ error }}</div>
      <footer><button type="button" @click="emit('close')">Cancel</button><button class="primary-button" :disabled="busy || !name.trim() || !value.trim()" type="submit">{{ busy ? 'Working…' : title }}</button></footer>
    </form>
  </div>
</template>
