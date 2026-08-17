import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useRepositoryStore } from "./repository";
import type { CommitRow } from "../types";

const apiMocks = vi.hoisted(() => ({
  workspace: vi.fn(), open: vi.fn(), log: vi.fn(), detail: vi.fn(), watch: vi.fn(),
}));
vi.mock("../lib/bridge", () => ({ api: apiMocks }));

function commit(subject: string, author: string, shortOid: string, decorations: string[] = []): CommitRow {
  return { oid: shortOid.padEnd(40, "0"), shortOid, parents: [], author, authorEmail: `${author}@example.test`, timestamp: 0, relativeDate: "now", subject, decorations };
}

describe("repository store", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.stubGlobal("localStorage", { getItem: vi.fn(), setItem: vi.fn() });
    vi.clearAllMocks();
  });

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

  it("loads details for the initially selected commit", async () => {
    const first = commit("Initial", "Kai", "abc1234");
    apiMocks.open.mockResolvedValue({ root: "/repo", name: "repo", branch: "main", changes: [], branches: [], tags: [], remotes: [], worktrees: [], state: {} });
    apiMocks.log.mockResolvedValue({ commits: [first], hasMore: false });
    apiMocks.detail.mockResolvedValue({ oid: first.oid, files: [{ status: "A", path: "README.md" }], patch: "+hello" });
    apiMocks.watch.mockResolvedValue(undefined);
    const store = useRepositoryStore();
    await store.loadRepository("/repo");
    expect(store.selectedCommit?.oid).toBe(first.oid);
    expect(store.detail?.files[0].path).toBe("README.md");
    expect(apiMocks.detail).toHaveBeenCalledWith("/repo", first.oid);
  });

  it("opens a mono workspace and restores its selected child repository", async () => {
    const first = commit("Initial", "Kai", "abc1234");
    apiMocks.workspace.mockResolvedValue({
      root: "/workspace", name: "workspace", kind: "monorepo",
      repositories: [
        { root: "/workspace/alpha", name: "alpha", branch: "main" },
        { root: "/workspace/beta", name: "beta", branch: "release" },
      ],
    });
    vi.mocked(localStorage.getItem).mockImplementation((key: string) => key === "graft.workspaceRepository:/workspace" ? "/workspace/beta" : null);
    apiMocks.open.mockResolvedValue({ root: "/workspace/beta", name: "beta", branch: "release", changes: [], branches: [], tags: [], remotes: [], worktrees: [], state: {} });
    apiMocks.log.mockResolvedValue({ commits: [first], hasMore: false });
    apiMocks.detail.mockResolvedValue({ oid: first.oid, files: [], patch: "" });
    apiMocks.watch.mockResolvedValue(undefined);

    const store = useRepositoryStore();
    await store.loadWorkspace("/workspace");

    expect(store.workspace?.kind).toBe("monorepo");
    expect(store.repository?.root).toBe("/workspace/beta");
    expect(apiMocks.open).toHaveBeenCalledWith("/workspace/beta");
    expect(localStorage.setItem).toHaveBeenCalledWith("graft.lastWorkspace", "/workspace");
  });
});
