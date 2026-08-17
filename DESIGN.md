---
name: Graft
description: A dense, native-feeling Git observatory for macOS.
colors:
  canvas: "oklch(0.965 0.004 260)"
  surface: "oklch(0.935 0.006 260)"
  surface-raised: "oklch(0.985 0.003 260)"
  surface-hover: "oklch(0.905 0.012 260)"
  surface-selected: "oklch(0.875 0.055 260)"
  separator: "oklch(0.80 0.009 260)"
  separator-soft: "oklch(0.865 0.007 260)"
  text: "oklch(0.235 0.012 260)"
  text-secondary: "oklch(0.37 0.015 260)"
  text-muted: "oklch(0.45 0.013 260)"
  cobalt: "oklch(0.49 0.18 260)"
  cobalt-strong: "oklch(0.42 0.19 260)"
  cobalt-text: "oklch(0.35 0.16 260)"
  amber: "oklch(0.59 0.14 72)"
  red: "oklch(0.53 0.19 25)"
  green: "oklch(0.49 0.12 155)"
  focus: "oklch(0.56 0.19 260 / 0.75)"
  on-cobalt: "#ffffff"
typography:
  title:
    fontFamily: '-apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif'
    fontSize: "14px"
    fontWeight: 700
    lineHeight: 1.35
  body:
    fontFamily: '-apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif'
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.35
  panel-title:
    fontFamily: '-apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif'
    fontSize: "13px"
    fontWeight: 590
    lineHeight: 1.35
  label:
    fontFamily: '-apple-system, BlinkMacSystemFont, "SF Pro Text", system-ui, sans-serif'
    fontSize: "10px"
    fontWeight: 400
    lineHeight: 1.35
  code:
    fontFamily: '"SFMono-Regular", "SF Mono", ui-monospace, monospace'
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.5
rounded:
  compact: "4px"
  control: "5px"
  field: "6px"
  floating: "7px"
  dialog: "12px"
spacing:
  tight: "4px"
  control: "7px"
  panel: "9px"
  dialog: "16px"
components:
  button-primary:
    backgroundColor: "{colors.cobalt}"
    textColor: "{colors.on-cobalt}"
    typography: "{typography.panel-title}"
    rounded: "{rounded.control}"
    padding: "0 12px"
    height: "28px"
  button-primary-hover:
    backgroundColor: "{colors.cobalt-strong}"
    textColor: "{colors.on-cobalt}"
    typography: "{typography.panel-title}"
    rounded: "{rounded.control}"
    padding: "0 12px"
    height: "28px"
  field-search:
    backgroundColor: "{colors.surface-raised}"
    textColor: "{colors.text}"
    typography: "{typography.body}"
    rounded: "{rounded.field}"
    padding: "0 7px"
    height: "28px"
  row-commit:
    backgroundColor: "{colors.canvas}"
    textColor: "{colors.text}"
    typography: "{typography.body}"
    rounded: "0"
    padding: "0 8px 0 4px"
    height: "31px"
  row-commit-selected:
    backgroundColor: "{colors.surface-selected}"
    textColor: "{colors.text}"
    typography: "{typography.body}"
    rounded: "0"
    padding: "0 8px 0 4px"
    height: "31px"
  tag-reference:
    backgroundColor: "transparent"
    textColor: "{colors.cobalt-text}"
    typography: "{typography.label}"
    rounded: "{rounded.compact}"
    padding: "0 5px"
    height: "17px"
---

# Design System: Graft

## Overview

**Creative North Star: "The Commit Observatory"**

Graft is a compact macOS instrument for reading repository topology and acting on it without leaving the current context. The interface is organized as one continuous workspace: native toolbar above, refs and worktrees at left, commit graph over diff in the center, commit preparation at right, and a narrow tool stripe on the far edge. Fine dividers, stable columns, and restrained state color make a large amount of Git information feel composed rather than ornamental.

The visual world ships in three selectable appearances: a native light gray default, a complete dark counterpart, and a Droid appearance built from warm graphite with an orange accent. The selected appearance is persistent and changes token values rather than component semantics. Cobalt marks selection, focus, the current branch, and the next primary action in Light and Dark; orange owns those same roles in Droid. Amber, red, and green report operational meaning. Lucide line icons support established labels and never replace important Git state on their own. Motion is short and local, while consequential flows expand into purpose-built dialogs for history operations, conflicts, hunks, references, and worktrees.

**Key Characteristics:**

- Four-pane, code-led workspace with IDEA-level information density.
- macOS system surfaces, compact controls, and crisp one-pixel separators.
- Real commit topology and monospace diffs as the visual center of gravity.
- Cobalt selection signal with explicit amber, red, and green operational states.
- Quiet Lucide line icons, keyboard focus, and reduced-motion support.

## Colors

The default light scheme uses cool alloy grays; the same semantic roles resolve to darker blue-grays under the system dark appearance.

### Primary

- **Instrument Cobalt:** Primary actions, current refs, active tabs, graph emphasis, selection details, and focus borders.
- **Deep Instrument Cobalt:** Hovered primary actions only.
- **Readable Cobalt:** Accent text and outlined reference labels that sit on neutral surfaces.

### Secondary

- **Operation Amber:** In-progress operations, modified-file state, exceptional limits, and caution.
- **Destructive Red:** Conflicts, validation errors, destructive actions, deletions, and irreversible warnings.
- **Clean Green:** Added lines, successful notices, and clean-working-tree confirmation.

### Neutral

- **Fog Canvas:** Main graph, detail, diff, editor, and dialog-field background.
- **Alloy Surface:** Sidebars, toolbars, headers, column bands, and the status bar.
- **Raised Paper:** Inputs, menus, dialogs, hunk containers, and transient notices.
- **Hover Steel:** Pointer feedback and quiet secondary actions.
- **Selected Cobalt Mist:** Selected rows, current refs, repository mark, and compact badges.
- **Structural Divider / Soft Divider:** Hard pane boundaries and subordinate row or section boundaries.
- **Graphite Ink / Secondary Ink / Muted Steel:** Primary content, supporting metadata, and low-priority counts or paths.

**The Rare Signal Rule.** Cobalt is reserved for selection, focus, current context, and the primary next action; it does not decorate neutral workspace chrome.

**The Semantic Redundancy Rule.** Amber, red, and green always accompany a label, file-state letter, icon, line sign, or action wording.

**The Appearance Role Rule.** Light, Dark, and Droid change token values, not their meaning; components continue to reference semantic roles.

### Appearance Themes

- **Light:** Cool alloy grays with Instrument Cobalt. This is the default and the visual baseline for daytime use.
- **Dark:** Cool blue-gray surfaces with a brighter cobalt accent for low-light, long-session work.
- **Droid:** Warm graphite surfaces with operational orange as the accent. It remains restrained and tool-like; orange replaces cobalt only for selection, focus, current context, graph emphasis, and the primary action.

Theme selection lives in the titlebar, persists across launches, and never changes density, layout, vocabulary, or semantic status colors.

## Typography

**Display Font:** System sans stack led by SF Pro Text
**Body Font:** System sans stack led by SF Pro Text
**Label/Mono Font:** SF Mono stack for code-bearing content

**Character:** One native sans family keeps the interface visually quiet, while the mono family makes hashes, paths, diff lines, and merge content mechanically legible. The ramp is deliberately compressed: most interface text sits between 9px and 14px, with weight doing more hierarchy work than size.

### Hierarchy

- **Title** (700, 14px, 1.35): Dialog titles and major operation headings.
- **Panel Title** (590, 13px, 1.35): Persistent pane headings and selected commit subjects.
- **Body** (400, 13px, 1.35): Base controls, commit subjects, form values, and primary workspace copy.
- **Metadata** (400–590, 9–12px, 1.35–1.5): Column headings, authors, dates, counters, statuses, shortcuts, and secondary guidance.
- **Code** (400, 10–11px, 1.45–1.5): Hashes, paths, patch text, hunk headers, rebase identifiers, and merge editors.

**The Native Density Rule.** Application typography stays compact and fixed-size; large display type is limited to the repository-empty welcome state.

**The Mechanical Content Rule.** Use monospace only when character alignment or exact source text carries meaning.

## Layout

The repository workspace fills the window and uses a fixed shell: a 52px titlebar, flexible working row, and 23px status bar. The titlebar establishes context from left to right: selected Workspace first, active branch second, workspace-scoped actions next, and terminal plus repository transport actions at the right. It never repeats the repository name in a centered title. Its default columns are a 222px repository sidebar, a center workspace with a 420px minimum, a 286px commit tool, and a 31px vertical tool stripe. The center divides into a 39px action toolbar, 23px column header, a commit graph that receives 46% of the remaining height, and a diff/detail panel that receives 54%.

Rows use an IDEA-like compact rhythm: commit and rebase rows are 31px, repository and change rows are 25–27px, and standard controls are 28–29px. Persistent panes touch edge to edge and are separated by one-pixel rules. Side panels collapse completely instead of becoming floating drawers, preserving the center graph as the main working surface.

At widths below 1100px, the sidebar narrows to 190px, the commit tool narrows to 250px, toolbar labels disappear, and the Author column is removed. This is a desktop adaptation, not a mobile transformation; minimum widths and horizontal scrolling preserve Git data instead of reflowing it into cards.

**The Four-Pane Continuity Rule.** Refs, history, detail, and commit preparation remain spatially adjacent so every action retains repository context.

**The Row Rhythm Rule.** Dense data uses stable row heights and aligned columns; variable-height cards do not replace repository lists.

## Elevation & Depth

The persistent workspace is flat. Tonal layers and structural separators establish depth, while the shared floating shadow is reserved for dialogs, menus, toasts, and loading indicators that sit above the workspace. Backdrops use translucent near-black to isolate consequential operations. Focus uses a crisp cobalt outline or inset ring rather than a decorative glow.

### Shadow Vocabulary

- **Floating Operation:** A two-stage, cool-black shadow (`0 18px 50px oklch(0.18 0.02 260 / 0.22), 0 3px 10px oklch(0.18 0.02 260 / 0.18)`) for dialogs, pop-up menus, toasts, and floating history feedback.

**The Flat Workspace Rule.** No persistent pane, toolbar, list, or diff receives a shadow.

**The Earned Elevation Rule.** A shadow means the element is transient and physically above the workspace.

## Shapes

Graft uses small radii that follow control scale: 4px for rows, tags, and compact actions; 5–6px for buttons and fields; 7px for menus and toasts; and 12px for modal operation surfaces. Persistent split panes remain square and edge-aligned. Borders are one-pixel structural lines; rounded outlines are localized to interactive controls and compact semantic containers.

**The Tool, Not Card Rule.** Information architecture comes from panes, columns, rows, and separators. Rounded containers are reserved for controls, hunk blocks, warnings, and floating operations.

## Components

### Buttons

- **Shape:** Standard actions use compact 5px corners and a 28px minimum height; icon-only controls are square 28px targets.
- **Primary:** Instrument Cobalt with white text, medium weight, and 12px horizontal padding. Split commit actions keep a continuous silhouette around the main action and menu trigger.
- **Hover / Focus:** Primary actions deepen to Deep Instrument Cobalt. Quiet actions pick up Hover Steel. Keyboard focus is a 2px cobalt outline inset by 2px; increased contrast widens it to 3px.
- **Secondary / Destructive:** Secondary dialog buttons use the Alloy Surface with a structural border. Confirmed destructive actions replace cobalt with Destructive Red and use an exact operation label.

### Chips

- **Style:** Reference labels are 17px high, transparent, softly outlined with a cobalt-to-divider mix, and set in 9px accent text. Current and status badges use Selected Cobalt Mist rather than a saturated fill.
- **State:** A Lucide or textual cue sits beside the color signal; chips describe repository state and are not decorative filters.

### Cards / Containers

- **Corner Style:** Persistent panes are square; localized hunk containers use 6px corners and transient menus or toasts use 7px corners.
- **Background:** Canvas supports code, Alloy Surface supports pane chrome, and Raised Paper supports fields and floating content.
- **Shadow Strategy:** Persistent containers are flat; only floating operations use the shared elevation token.
- **Border:** One-pixel Structural Divider for pane boundaries and Soft Divider inside a component.
- **Internal Padding:** Compact containers generally use 7–9px; dialogs use 16px side insets.

### Inputs / Fields

- **Style:** Search and commit fields sit on Raised Paper with a one-pixel divider stroke and 5–6px corners. Standard fields are 28–29px high; the commit message editor is 86px high and vertically resizable.
- **Focus:** The divider turns cobalt and gains a one-pixel focus ring; full merge-result editors use a two-pixel inset ring.
- **Error / Disabled:** Errors remain inline in Destructive Red. Disabled controls retain their structure at 56% opacity.

### Navigation

Repository navigation begins directly below the titlebar and uses 25–27px tree rows with disclosure chevrons, Lucide line icons, counts, and nested 18px indentation. A regular Workspace begins with Branches; a Mono Repo begins with a Repositories group whose selected child determines the Branches, Remotes, Tags, Worktrees, history, diff, and terminal path shown elsewhere. Hover uses Hover Steel; the current branch uses Selected Cobalt Mist plus a cobalt icon and the word “current.” The far-right tool stripe rotates labels vertically, keeps icons upright, and marks the active tool with a 2px cobalt edge bar.

### Commit Graph & Diff

The commit log is a virtualized 31px row grid with a fixed 72px SVG topology lane and aligned subject, decoration, author, hash, and date columns. Selection changes the full row surface. The graph is supplemental to accessible DOM text. Diffs use SF Mono, persistent line numbers, and low-chroma green/red/cobalt line washes; addition, deletion, and hunk syntax remains visible through signs and labels as well as color.

### Operation Dialogs

Routine creation and history actions use compact 480px dialogs; interactive rebase, hunk selection, conflict resolution, and multi-repository worktree creation expand into wide, task-specific workspaces. The Workspace Worktrees dialog exposes repository scope, starting-point policy, target directory, progress, and per-repository outcomes in one surface; partial retries select only failed repositories. Dialogs retain the same surface roles, control scale, field behavior, and exact Git vocabulary as the main shell. Destructive scope appears inline before the action and changes the confirmation label and color.

## Do's and Don'ts

### Do:

- **Do** keep the real repository topology, selected commit, changed files, diff, and commit preparation visible together.
- **Do** preserve the 25–31px row rhythm, aligned metadata columns, truncation, and scrollable code surfaces for large data.
- **Do** use semantic surface tokens and let system appearance swap their values.
- **Do** pair Lucide line icons and semantic colors with explicit labels, letters, counts, or action wording.
- **Do** make dangerous operations deliberate with visible scope, precise Git terms, and recovery guidance.
- **Do** retain focus, increased-contrast, and reduced-motion behavior on every new control or transition.
- **Do** keep Workspace, active child repository, active branch, and terminal target visibly consistent across every pane and action.

### Don't:

- **Don't** turn repository content into a generic dashboard of detached cards or summary tiles.
- **Don't** add decorative glass, broad gradients, oversized rounding, or shadows to persistent panes.
- **Don't** use cobalt as ambient branding; save it for selection, focus, current context, and primary action.
- **Don't** hide Git state, rename established operations, or use color as the only status cue.
- **Don't** replace dense desktop columns with a mobile card stack; remove lower-priority metadata before changing the interaction model.
- **Don't** introduce ornamental typefaces, display-scale headings, emoji, or glyph icons into the working interface.
