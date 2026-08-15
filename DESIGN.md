<!-- SEED: re-run $impeccable document once there's code to capture the actual tokens and components. -->
---
name: Graft
description: A precise, native-feeling Git client for macOS.
---

# Design System: Graft

## Overview

**Creative North Star: "The Commit Observatory"**

Graft is a dense instrument used for hours at a time. Its visual system should recall a macOS observatory before dawn: neutral working surfaces, crisp hierarchy, and a restrained cobalt signal used only where attention or selection matters. The information architecture follows the proven shape of Rebased and IDEA, while the finish takes cues from focused macOS system tools and Xcode rather than from browser dashboards.

Motion is responsive but quiet. State changes may crossfade or slide over 150–200 ms, while loading, list virtualization, and graph updates never become choreography. The interface rejects decorative glass, oversized cards, gratuitous gradients, and IDE chrome unrelated to Git.

**Key Characteristics:**

- Dense, aligned, and keyboard-first.
- Neutral surfaces with one rare cobalt selection signal.
- Stable graph colors that remain distinguishable without being fluorescent.
- Explicit focus, progress, disabled, warning, destructive, and recovery states.
- Native macOS familiarity without imitating AppKit pixel for pixel.

## Colors

The strategy is restrained: achromatic system surfaces and a pre-dawn cobalt anchor occupying no more than ten percent of a screen. Exact light and dark OKLCH tokens will be resolved during implementation and verified against macOS increased-contrast settings.

### Primary

- **Instrument Cobalt** (`oklch(0.450 0.150 260)` seed): selection, current branch, focused graph lane, and primary actions only.

### Secondary

- **Calibration Amber** (`[to be resolved during implementation]`): warnings and exceptional attention states, never decoration.

### Neutral

- **System Canvas** (`[to be resolved during implementation]`): primary content background, derived from the current system appearance.
- **Panel Alloy** (`[to be resolved during implementation]`): sidebars, toolbars, and inspector surfaces.
- **Graphite Ink** (`[to be resolved during implementation]`): primary text at a minimum 7:1 contrast against the canvas.
- **Muted Steel** (`[to be resolved during implementation]`): secondary metadata at a minimum 4.5:1 contrast.

**The Rare Signal Rule.** Instrument Cobalt occupies at most ten percent of the visible surface. It communicates selection or action, never brand decoration.

**The Semantic Redundancy Rule.** File state, graph lanes, conflicts, success, and failure always have a non-color cue.

## Typography

**Display Font:** SF Pro / system-ui
**Body Font:** SF Pro / system-ui
**Label/Mono Font:** SF Mono / ui-monospace

**Character:** A single native sans keeps controls and dense metadata familiar. Monospace is reserved for hashes, paths, patches, command output, and commit message editing where alignment carries meaning.

### Hierarchy

- **Headline** (600, 17px, 1.25): window-level or major empty-state titles only.
- **Title** (600, 13px, 1.3): panel headers, selected commit subjects, and dialog titles.
- **Body** (400, 13px, 1.4): lists, forms, messages, and diff metadata.
- **Label** (500, 11px, normal case): compact metadata and toolbar controls.
- **Code** (400, 12px, 1.45): hashes, paths, patches, and editable merge content.

**The Native Density Rule.** Type remains fixed-size and compact. Never use fluid display typography or marketing-page scale inside the application shell.

## Elevation

Graft is flat by default. Depth comes from tonal layering, separators, and macOS window hierarchy. Shadows are reserved for transient content that physically floats above the workspace, such as menus, popovers, and command palettes.

**The Flat Workspace Rule.** Persistent panels never use decorative shadows. If every pane looks like a card, the hierarchy has failed.

## Components

This seed intentionally defers exact component tokens until the first running interface exists. Components must use standard macOS affordances, compact hit targets of at least 28px visually with accessible pointer areas where needed, and complete default, hover, focus, active, disabled, loading, error, and destructive states.

Buttons use specific verb-and-object labels. Lists preserve selection while details load. Context menus use IDEA-compatible naming, grouping, order, and keyboard behavior. Split panes expose keyboard resizing and remember their sizes. The commit graph keeps text in accessible DOM rows and treats its canvas or SVG as supplemental presentation.

## Do's and Don'ts

### Do:

- **Do** preserve IDEA-compatible control placement, focus order, shortcuts, and operation feedback.
- **Do** use compact rows, stable column alignment, skeleton loading, and retained selection.
- **Do** respect light, dark, increased contrast, reduced transparency, and reduced motion.
- **Do** preview destructive scope and name the exact operation on confirmation buttons.
- **Do** reserve visual emphasis for repository state and the user's current task.

### Don't:

- **Don't** build a generic web administration dashboard inside a desktop window.
- **Don't** use decorative glass panels, oversized cards, excessive rounding, gradients, or animation without operational meaning.
- **Don't** simplify Git by hiding state, renaming established operations, or forcing a novel workflow.
- **Don't** reproduce IDE chrome or feature density unrelated to Git.
- **Don't** imitate a terminal at the expense of discoverability or accessibility.
- **Don't** use color as the only indication of graph lanes, file state, conflicts, or operation outcome.
