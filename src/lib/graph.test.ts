import { describe, expect, it } from "vitest";
import { allocateGraphRows } from "./graph";
import type { CommitRow } from "../types";

const row = (oid: string, parents: string[]): CommitRow => ({ oid, shortOid: oid, parents, author: "A", authorEmail: "a@test", timestamp: 0, relativeDate: "now", subject: oid, decorations: [] });

describe("commit graph lane allocation", () => {
  it("tracks a branch split and merge by parent identity", () => {
    const graph = allocateGraphRows([
      row("merge", ["left", "right"]),
      row("right", ["base"]),
      row("left", ["base"]),
      row("base", []),
    ]);
    expect(graph[0].parentLanes).toEqual([0, 1]);
    expect(graph[1].lane).toBe(1);
    expect(graph[2].lane).toBe(0);
    expect(graph[3].lane).toBe(0);
  });

  it("caps presentation lanes without losing commit rows", () => {
    const commits = Array.from({ length: 12 }, (_, index) => row(`head-${index}`, []));
    expect(allocateGraphRows(commits, 4)).toHaveLength(12);
    expect(Math.max(...allocateGraphRows(commits, 4).map((item) => item.lane))).toBeLessThan(4);
  });
});

