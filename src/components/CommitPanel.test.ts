import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import CommitPanel from "./CommitPanel.vue";
import type { CommitDetail, CommitRow } from "../types";

const commit: CommitRow = {
  oid: "abcdef1234567890",
  shortOid: "abcdef1",
  parents: ["parent"],
  author: "Kai",
  authorEmail: "kai@example.test",
  timestamp: 0,
  relativeDate: "now",
  subject: "Refine details",
  decorations: [],
};

const detail: CommitDetail = {
  oid: commit.oid,
  author: commit.author,
  authorEmail: commit.authorEmail,
  authoredAt: "2026-08-16T00:00:00Z",
  committer: commit.author,
  committedAt: "2026-08-16T00:00:00Z",
  subject: commit.subject,
  body: "",
  parents: commit.parents,
  files: [{ status: "M", path: "README.md" }, { status: "A", path: "src/new.ts" }],
  patch: [
    "diff --git a/README.md b/README.md",
    "--- a/README.md",
    "+++ b/README.md",
    "@@ -1 +1 @@",
    "-old readme",
    "+new readme",
    "diff --git a/src/new.ts b/src/new.ts",
    "--- /dev/null",
    "+++ b/src/new.ts",
    "@@ -0,0 +1 @@",
    "+export const ready = true;",
  ].join("\n"),
};

describe("CommitPanel", () => {
  it("filters the preview to a selected changed file and restores all files", async () => {
    const wrapper = mount(CommitPanel, { props: { commit, detail } });
    expect(wrapper.find(".patch").text()).toContain("old readme");
    expect(wrapper.find(".patch").text()).toContain("ready = true");

    const fileButtons = wrapper.findAll(".detail-file");
    await fileButtons[2].trigger("click");
    expect(wrapper.find(".patch").text()).not.toContain("old readme");
    expect(wrapper.find(".patch").text()).toContain("ready = true");

    await fileButtons[0].trigger("click");
    expect(wrapper.find(".patch").text()).toContain("old readme");
  });
});
