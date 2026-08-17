export interface FilePatch {
  path: string;
  patch: string;
}

const unquoteGitPath = (value: string) => {
  const trimmed = value.trim();
  if (!(trimmed.startsWith('"') && trimmed.endsWith('"'))) return trimmed;
  try {
    return JSON.parse(trimmed) as string;
  } catch {
    return trimmed.slice(1, -1);
  }
};

function sectionPath(section: string): string {
  const rename = section.match(/^rename to (.+)$/m)?.[1];
  if (rename) return unquoteGitPath(rename);

  const added = section.match(/^\+\+\+ (.+)$/m)?.[1];
  if (added && added !== "/dev/null") {
    const path = unquoteGitPath(added);
    return path.startsWith("b/") ? path.slice(2) : path;
  }

  const deleted = section.match(/^--- (.+)$/m)?.[1];
  if (deleted && deleted !== "/dev/null") {
    const path = unquoteGitPath(deleted);
    return path.startsWith("a/") ? path.slice(2) : path;
  }

  return "";
}

/** Classifies a unified-diff line for styling; headers keep the neutral color. */
export function diffLineClass(line: string): "addition" | "deletion" | "hunk" | "" {
  if (line.startsWith("+") && !line.startsWith("+++")) return "addition";
  if (line.startsWith("-") && !line.startsWith("---")) return "deletion";
  if (line.startsWith("@@")) return "hunk";
  return "";
}

export function splitCommitPatch(patch: string): FilePatch[] {
  if (!patch.trim()) return [];
  const starts: number[] = [];
  const marker = /^diff --git /gm;
  for (let match = marker.exec(patch); match; match = marker.exec(patch)) starts.push(match.index);
  if (!starts.length) return [{ path: "", patch }];

  return starts.map((start, index) => {
    const section = patch.slice(start, starts[index + 1] ?? patch.length);
    return { path: sectionPath(section), patch: section };
  });
}
