import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it } from "vitest";
import { useRepositoryStore } from "./repository";
import type { CommitRow } from "../types";

function commit(subject: string, author: string, shortOid: string, decorations: string[] = []): CommitRow {
  return { oid: shortOid.padEnd(40, "0"), shortOid, parents: [], author, authorEmail: `${author}@example.test`, timestamp: 0, relativeDate: "now", subject, decorations };
}

describe("repository store", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("filters loaded commits by message, author, hash, and ref", () => {
    const store = useRepositoryStore();
    store.commits = [commit("Fix worktree pruning", "Kai", "abc1234", ["main"]), commit("Update docs", "Mira", "def5678", ["release/v1"])] as never;
    store.query = "worktree";
    expect(store.visibleCommits.map((item) => item.shortOid)).toEqual(["abc1234"]);
    store.query = "release/v1";
    expect(store.visibleCommits.map((item) => item.shortOid)).toEqual(["def5678"]);
    store.query = "KAI";
    expect(store.visibleCommits.map((item) => item.shortOid)).toEqual(["abc1234"]);
  });
});

