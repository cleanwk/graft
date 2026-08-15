import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import CommitToolWindow from "./CommitToolWindow.vue";
import type { Change } from "../types";

const changes: Change[] = [
  { path: "src/staged.ts", indexStatus: "M", worktreeStatus: ".", staged: true, conflicted: false },
  { path: "src/working.ts", indexStatus: ".", worktreeStatus: "M", staged: false, conflicted: false },
  { path: "src/conflict.ts", indexStatus: "U", worktreeStatus: "U", staged: false, conflicted: true },
];

describe("CommitToolWindow", () => {
  it("keeps IDEA-style included and working change groups", () => {
    const wrapper = mount(CommitToolWindow, { props: { changes } });
    expect(wrapper.text()).toContain("Included in commit");
    expect(wrapper.text()).toContain("src/staged.ts");
    expect(wrapper.text()).toContain("src/working.ts");
    expect(wrapper.text()).toContain("Resolve…");
    expect(wrapper.find(".change-groups section:first-of-type").text()).not.toContain("src/conflict.ts");
    expect(wrapper.find(".change-groups section:last-of-type").text()).toContain("src/conflict.ts");
    expect(wrapper.get("button[type=submit]").attributes("disabled")).toBeDefined();
  });

  it("moves whole files and opens conflict resolution", async () => {
    const wrapper = mount(CommitToolWindow, { props: { changes } });
    const unchecked = wrapper.findAll<HTMLInputElement>('input[type="checkbox"]').find((input) => !input.element.checked)!;
    await unchecked.setValue(true);
    expect(wrapper.emitted("staged")?.[0]).toEqual(["src/working.ts", true]);
    await wrapper.get(".resolve-link").trigger("click");
    expect(wrapper.emitted("resolve")?.[0]).toEqual(["src/conflict.ts"]);
  });
});
