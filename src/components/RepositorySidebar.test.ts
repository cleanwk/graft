import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import RepositorySidebar from "./RepositorySidebar.vue";
import type { RepositorySnapshot } from "../types";

const repository: RepositorySnapshot = {
  root: "/repos/graft",
  name: "graft",
  branch: "main",
  head: "abc1234",
  ahead: 0,
  behind: 0,
  state: { merging: false, rebasing: false, cherryPicking: false, reverting: false, conflicts: 0 },
  changes: [],
  changesTruncated: false,
  branches: [
    { name: "main", current: true, remote: false, oid: "abc1234" },
    { name: "origin", current: false, remote: true, oid: "abc1234" },
    { name: "origin/main", current: false, remote: true, oid: "abc1234" },
    { name: "origin/feature/topic", current: false, remote: true, oid: "def5678" },
    { name: "upstream/edge", current: false, remote: true, oid: "9876abc" },
  ],
  tags: ["v1.0.0"],
  remotes: ["origin", "upstream"],
  worktrees: [],
};

describe("RepositorySidebar", () => {
  it("nests remote branches under their remote and skips HEAD symrefs", () => {
    const wrapper = mount(RepositorySidebar, { props: { repository } });
    const nested = wrapper.findAll(".tree-nested .tree-static");
    expect(nested.map((row) => row.text())).toEqual(["main", "feature/topic", "edge"]);
  });

  it("does not emit checkout for the current branch", async () => {
    const wrapper = mount(RepositorySidebar, { props: { repository } });
    await wrapper.get(".tree-items button").trigger("dblclick");
    expect(wrapper.emitted("checkout")).toBeUndefined();
  });
});
