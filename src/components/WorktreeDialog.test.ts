import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import WorktreeDialog from "./WorktreeDialog.vue";
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
    { name: "feature/topic", current: false, remote: false, oid: "def5678" },
    { name: "review", current: false, remote: false, oid: "9876abc" },
  ],
  tags: [],
  remotes: ["origin"],
  worktrees: [{ path: "/repos/graft", branch: "main", head: "abc1234", bare: false, locked: false, prunable: false }],
};

describe("WorktreeDialog", () => {
  it("starts on an unused branch and keeps the suggested path in sync", async () => {
    const wrapper = mount(WorktreeDialog, { props: { repository } });
    const branch = wrapper.get<HTMLSelectElement>('select[aria-label="Worktree branch"]');
    expect(branch.element.value).toBe("feature/topic");
    expect(wrapper.get<HTMLInputElement>('.path-field input').element.value).toBe("/repos/graft-worktrees/feature-topic");
    expect(wrapper.findAll("option")[0].attributes("disabled")).toBeDefined();

    await branch.setValue("review");
    expect(wrapper.get<HTMLInputElement>('.path-field input').element.value).toBe("/repos/graft-worktrees/review");
  });

  it("filters worktrees and branch choices from one search field", async () => {
    const wrapper = mount(WorktreeDialog, { props: { repository } });
    await wrapper.get<HTMLInputElement>('input[aria-label="Search worktrees and branches"]').setValue("review");

    expect(wrapper.text()).toContain("No matching worktrees.");
    expect(wrapper.findAll("option").map((option) => option.text())).toEqual(["review"]);
  });
});
