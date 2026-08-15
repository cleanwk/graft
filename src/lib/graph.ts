import type { CommitRow } from "../types";

export interface GraphRow { lane: number; before: number; nextLanes: number[]; parentLanes: number[] }

export function allocateGraphRows(commits: CommitRow[], maxLanes = 8): GraphRow[] {
  const lanes: string[] = [];
  return commits.map((commit) => {
    let lane = lanes.indexOf(commit.oid);
    if (lane < 0) { lane = Math.min(lanes.length, maxLanes - 1); lanes.splice(lane, 0, commit.oid); }
    const before = Math.min(lanes.length, maxLanes);
    const firstParent = commit.parents[0];
    if (firstParent) lanes[lane] = firstParent; else lanes.splice(lane, 1);
    commit.parents.slice(1).forEach((parent, offset) => { if (!lanes.includes(parent)) lanes.splice(lane + 1 + offset, 0, parent); });
    for (let index = lanes.length - 1; index >= 0; index--) if (lanes.indexOf(lanes[index]) !== index) lanes.splice(index, 1);
    const parentLanes = commit.parents.map((parent) => lanes.indexOf(parent)).filter((value) => value >= 0 && value < maxLanes);
    return { lane, before, nextLanes: lanes.slice(0, maxLanes).map((_, index) => index), parentLanes };
  });
}

