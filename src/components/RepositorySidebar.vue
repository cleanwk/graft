<script setup lang="ts">
import { ref } from "vue";
import { ChevronRight, FolderGit2, GitBranch, GitFork, Plus, Radio, Tags, TreePine } from "@lucide/vue";
import type { RepositorySnapshot } from "../types";

defineProps<{ repository: RepositorySnapshot }>();
const emit = defineEmits<{ addWorktree: [] }>();
const sections = ref({ branches: true, remotes: true, tags: false, worktrees: true });
</script>

<template>
  <aside class="repo-sidebar" aria-label="Repository navigation">
    <header class="repo-heading">
      <span class="repo-icon"><FolderGit2 :size="15" /></span>
      <div><strong>{{ repository.name }}</strong><span>{{ repository.root }}</span></div>
    </header>
    <nav>
      <section>
        <button class="tree-heading" @click="sections.branches = !sections.branches"><ChevronRight :size="12" :class="{ expanded: sections.branches }" /><GitBranch :size="13" /><span>Branches</span><b>{{ repository.branches.filter(b => !b.remote).length }}</b></button>
        <div v-if="sections.branches" class="tree-items">
          <button v-for="branch in repository.branches.filter(b => !b.remote).slice(0, 30)" :key="branch.name" :class="{ current: branch.current }">
            <GitBranch :size="12" /><span>{{ branch.name }}</span><i v-if="branch.current">current</i>
          </button>
        </div>
      </section>
      <section>
        <button class="tree-heading" @click="sections.remotes = !sections.remotes"><ChevronRight :size="12" :class="{ expanded: sections.remotes }" /><Radio :size="13" /><span>Remotes</span><b>{{ repository.remotes.length }}</b></button>
        <div v-if="sections.remotes" class="tree-items">
          <button v-for="remote in repository.remotes" :key="remote"><GitFork :size="12" /><span>{{ remote }}</span></button>
        </div>
      </section>
      <section>
        <button class="tree-heading" @click="sections.tags = !sections.tags"><ChevronRight :size="12" :class="{ expanded: sections.tags }" /><Tags :size="13" /><span>Tags</span><b>{{ repository.tags.length }}</b></button>
        <div v-if="sections.tags" class="tree-items"><button v-for="tag in repository.tags.slice(0, 30)" :key="tag"><Tags :size="12" /><span>{{ tag }}</span></button></div>
      </section>
      <section>
        <button class="tree-heading" @click="sections.worktrees = !sections.worktrees"><ChevronRight :size="12" :class="{ expanded: sections.worktrees }" /><TreePine :size="13" /><span>Worktrees</span><b>{{ repository.worktrees.length }}</b><Plus class="tree-action" :size="13" @click.stop="emit('addWorktree')" /></button>
        <div v-if="sections.worktrees" class="tree-items worktrees">
          <button v-for="worktree in repository.worktrees" :key="worktree.path" :title="worktree.path">
            <TreePine :size="12" /><span>{{ worktree.branch ?? 'detached' }}</span><i>{{ worktree.head }}</i>
          </button>
        </div>
      </section>
    </nav>
  </aside>
</template>
