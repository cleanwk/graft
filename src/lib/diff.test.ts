import { describe, expect, it } from "vitest";
import { splitCommitPatch } from "./diff";

describe("splitCommitPatch", () => {
  it("groups a combined commit patch by destination file", () => {
    const patch = [
      "diff --git a/README.md b/README.md",
      "--- a/README.md",
      "+++ b/README.md",
      "@@ -1 +1 @@",
      "-old",
      "+new",
      "diff --git a/src/old.ts b/src/new.ts",
      "similarity index 90%",
      "rename from src/old.ts",
      "rename to src/new.ts",
    ].join("\n");

    const files = splitCommitPatch(patch);
    expect(files.map((file) => file.path)).toEqual(["README.md", "src/new.ts"]);
    expect(files[0].patch).toContain("+new");
    expect(files[1].patch).toContain("rename to src/new.ts");
  });

  it("keeps an unstructured bounded message visible", () => {
    expect(splitCommitPatch("[Diff truncated]")).toEqual([{ path: "", patch: "[Diff truncated]" }]);
  });
});
