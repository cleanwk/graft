# Graft performance spike baseline

Date: 2026-08-15 (Asia/Shanghai)

## Environment

- Apple Silicon (`arm64`)
- macOS 27.0 preview, build 26A5388g
- Xcode 26.3
- Apple Git 2.50.1
- Rust 1.92.0
- Node 25.6.1
- Tauri 2.11.5, WKWebView
- Rebased 1.1.13 arm64, Homebrew cask SHA-256
  `788e7029e5a8b58cb419adcd24653bdffd72a126775c337929944a365ca150c5`

## Results

The first valid non-trivial fixture was `/Users/kai/github/codex` with 17,198
commits. The supplied local `rebased` checkout is shallow and contains one
commit, so a blob-filtered full-history reference clone is being prepared under
the ignored `.benchmarks/` directory.

| Measurement | Graft Tauri debug spike | Rebased 1.1.13 |
| --- | ---: | ---: |
| Parsed commits | 17,198 | 1 (local shallow checkout) |
| Git log command | 389 ms | not exposed |
| Window usable | 543 ms | not yet instrumented |
| Total related RSS | 154.98 MiB | 488.45 MiB |

Graft's total includes the application, WebKit Networking, GPU, Web Content,
and AppKit theme service. Rebased includes its main process and `fsnotifier`.
The Rebased figure was sampled after the project became trusted and its Git
tool window initialized. These figures are directional, not yet a controlled
same-repository comparison.

## Findings

1. Tauri's fixed empty-window cost is about 63 MiB total RSS on this machine.
2. Loading all 17,198 commit records in one JSON response raises total RSS to
   about 155 MiB. A 100k history must use paged or channel-based transfer with a
   bounded client cache; one giant JSON response is rejected for production.
3. TanStack Virtual mounts only the visible rows and keeps the rendered list
   responsive at 17k entries.
4. The initial TypeScript 7 selection was incompatible with the current
   `vue-tsc`; the verified toolchain is pinned to TypeScript 5.9.3.
5. Vite 8 no longer bundles the old esbuild minifier path by default. Boolean
   minification uses its supported current pipeline and avoids an unnecessary
   `esbuild` dependency.

## Evidence

- [Initial shallow-repository render](../acceptance/spike-initial.png)
- [17,198-commit render](../acceptance/spike-17198-commits.png)

## Next gate

Repeat the same measurements on the full Rebased history, replace the monolithic
IPC payload with bounded pages, exercise continuous scroll, and record dropped
frames plus peak/steady RSS before promoting any spike code to production.

## Production follow-up

The production implementation completed that gate on the full-history Rebased
repository (524,408 commits, 1.0 GiB blob-filtered clone):

- history is paged in 500-commit windows and virtualized in the UI;
- working-tree status is capped at 2,000 entries;
- commit patches are capped at 2 MiB before IPC serialization;
- the ignored production benchmark completed its measured body in 1.45 seconds;
- the test process and its unusually expensive Apple Git `status` child reached
  396 MiB maximum RSS; the retained release application, including WebKit helper
  processes, measured 232.5 MiB on a real repository with 500 displayed commits,
  18 changes, 19 branches, 195 tags, and 7 worktrees.

The release application starts registering with LaunchServices in roughly 266 ms
and completed the measured macOS application check-in in 1.61 seconds. These are
local measurements on the environment above, not cross-machine guarantees.

## 2026-08-17 memory containment pass

The 500 MiB report was consistent with the remaining unbounded client behavior:
virtualization limited mounted DOM rows, but repeated pagination still retained
every loaded commit and Vue recursively proxied the large repository, history,
and patch payloads. File-watcher events could also overlap refresh requests.

The client now requests 250 commits at a time and retains at most 2,000, stores
large immutable IPC responses in shallow refs, coalesces refreshes, caps status
at 500 entries, caps backend patch IPC at 512 KiB, and renders at most 1,000
patch lines. On the same real repository shape used above (500 displayed commits,
18 changes, 19 branches, 196 tags, and 7 worktrees), the rebuilt release process
set measured 217.5 MiB RSS immediately after restore: 94.8 MiB application,
70.9 MiB Web Content, 39.7 MiB GPU, and 12.1 MiB Networking. That is 15.0 MiB
(6.5%) below the earlier 232.5 MiB retained-release sample. The more important
long-session result is bounded growth: history and patch retention can no longer
continue expanding toward the 500 MiB report without limit.
