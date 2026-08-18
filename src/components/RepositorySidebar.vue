<script setup lang="ts">
import { computed, ref } from "vue";
import { Boxes, ChevronRight, GitBranch, GitFork, Plus, Radio, Tags, TreePine } from "@lucide/vue";
import type { RepositorySnapshot, WorkspaceSnapshot } from "../types";

const LIST_LIMIT = 30;

const props = defineProps<{ repository: RepositorySnapshot; workspace?: WorkspaceSnapshot }>();
const emit = defineEmits<{ addWorktree: []; addReference: [kind: "branch" | "tag" | "remote"]; checkout: [branch: string]; openWorktree: [path: string]; selectRepository: [path: string] }>();
const sections = ref({ repositories: true, branches: true, remotes: true, tags: false, worktrees: true });

const localBranches = computed(() => props.repository.branches.filter((branch) => !branch.remote));
const visibleBranches = computed(() => localBranches.value.slice(0, LIST_LIMIT));
const hiddenBranchCount = computed(() => localBranches.value.length - visibleBranches.value.length);
const visibleTags = computed(() => props.repository.tags.slice(0, LIST_LIMIT));
const hiddenTagCount = computed(() => props.repository.tags.length - visibleTags.value.length);
const remoteBranches = (remote: string) =>
  props.repository.branches.filter((branch) => branch.remote && branch.name.startsWith(`${remote}/`) && !branch.name.endsWith("/HEAD"));
const shortRemoteBranch = (name: string, remote: string) => name.slice(remote.length + 1);
</script>

<template>
  <aside class="repo-sidebar" aria-label="Repository navigation">
    <nav>
      <section v-if="workspace?.kind === 'monorepo'" class="workspace-repositories">
        <div class="tree-heading-row"><button class="tree-heading" :aria-expanded="sections.repositories" @click="sections.repositories = !sections.repositories"><ChevronRight :size="12" :class="{ expanded: sections.repositories }" /><Boxes :size="13" /><span>Repositories</span><b>{{ workspace.repositories.length }}</b></button></div>
        <div v-if="sections.repositories" class="tree-items">
          <button v-for="item in workspace.repositories" :key="item.root" :class="{ current: item.root === repository.root }" :title="`${item.root} — select repository`" @click="emit('selectRepository', item.root)">
            <Boxes :size="12" /><span>{{ item.name }}</span><i v-if="item.root === repository.root">selected</i>
          </button>
        </div>
      </section>
      <section>
        <div class="tree-heading-row"><button class="tree-heading" :aria-expanded="sections.branches" @click="sections.branches = !sections.branches"><ChevronRight :size="12" :class="{ expanded: sections.branches }" /><GitBranch :size="13" /><span>Branches</span><b>{{ localBranches.length }}</b></button><button class="tree-action-button" aria-label="Create branch" title="Create branch" @click="emit('addReference', 'branch')"><Plus :size="13" /></button></div>
        <div v-if="sections.branches" class="tree-items">
          <button v-for="branch in visibleBranches" :key="branch.name" :class="{ current: branch.current }" :title="branch.current ? `${branch.name} — current branch` : `${branch.name} — double-click to check out`" @dblclick="!branch.current && emit('checkout', branch.name)">
            <GitBranch :size="12" /><span>{{ branch.name }}</span><i v-if="branch.current">current</i>
          </button>
          <div v-if="hiddenBranchCount > 0" class="tree-static tree-more"><span>…and {{ hiddenBranchCount }} more</span></div>
        </div>
      </section>
      <section>
        <div class="tree-heading-row"><button class="tree-heading" :aria-expanded="sections.remotes" @click="sections.remotes = !sections.remotes"><ChevronRight :size="12" :class="{ expanded: sections.remotes }" /><Radio :size="13" /><span>Remotes</span><b>{{ repository.remotes.length }}</b></button><button class="tree-action-button" aria-label="Add remote" title="Add remote" @click="emit('addReference', 'remote')"><Plus :size="13" /></button></div>
        <div v-if="sections.remotes" class="tree-items">
          <div v-for="remote in repository.remotes" :key="remote">
            <div class="tree-static"><GitFork :size="12" /><span>{{ remote }}</span></div>
            <div class="tree-items tree-nested">
              <div v-for="branch in remoteBranches(remote)" :key="branch.name" class="tree-static" :title="branch.name"><GitBranch :size="12" /><span>{{ shortRemoteBranch(branch.name, remote) }}</span></div>
            </div>
          </div>
        </div>
      </section>
      <section>
        <div class="tree-heading-row"><button class="tree-heading" :aria-expanded="sections.tags" @click="sections.tags = !sections.tags"><ChevronRight :size="12" :class="{ expanded: sections.tags }" /><Tags :size="13" /><span>Tags</span><b>{{ repository.tags.length }}</b></button><button class="tree-action-button" aria-label="Create tag" title="Create tag" @click="emit('addReference', 'tag')"><Plus :size="13" /></button></div>
        <div v-if="sections.tags" class="tree-items"><div v-for="tag in visibleTags" :key="tag" class="tree-static"><Tags :size="12" /><span>{{ tag }}</span></div><div v-if="hiddenTagCount > 0" class="tree-static tree-more"><span>…and {{ hiddenTagCount }} more</span></div></div>
      </section>
      <section>
        <div class="tree-heading-row"><button class="tree-heading" :aria-expanded="sections.worktrees" @click="sections.worktrees = !sections.worktrees"><ChevronRight :size="12" :class="{ expanded: sections.worktrees }" /><TreePine :size="13" /><span>Worktrees</span><b>{{ repository.worktrees.length }}</b></button><button class="tree-action-button" aria-label="Add worktree" title="Add worktree" @click="emit('addWorktree')"><Plus :size="13" /></button></div>
        <div v-if="sections.worktrees" class="tree-items worktrees">
          <button v-for="worktree in repository.worktrees" :key="worktree.path" :title="`${worktree.path} — double-click to open in a new window`" @dblclick="emit('openWorktree', worktree.path)">
            <TreePine :size="12" /><span>{{ worktree.branch ?? 'detached' }}</span><i>{{ worktree.head }}</i>
          </button>
        </div>
      </section>
    </nav>
  </aside>
</template>
