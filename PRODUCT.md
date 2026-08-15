# Product

## Register

product

## Users

Graft is for experienced macOS developers who already work fluently with the Git tools in IntelliJ IDEA or Rebased. They spend long sessions reviewing history, preparing precise commits, moving between branches and worktrees, and recovering from complicated operations. They expect dense information, complete keyboard control, and predictable Git semantics.

## Product Purpose

Graft is a fast, memory-conscious Git desktop client for macOS on Apple Silicon. It preserves the interaction habits that make IntelliJ's Git tooling productive while replacing the weight of an IDE platform with a focused Rust, Tauri, and Vue application. Success means a developer can complete their daily Git workflow, including worktrees, conflict resolution, and interactive rebase, without returning to IDEA or Rebased for a missing core operation.

## Brand Personality

Precise, composed, capable. Graft should feel like a serious native macOS instrument: quiet while work is routine, explicit when an operation is consequential, and never cute, theatrical, or chatty.

## Anti-references

- Generic web administration dashboards transplanted into a desktop window.
- Decorative glass panels, oversized cards, excessive rounding, gradients, or animation without operational meaning.
- Simplified Git clients that hide state, rename established operations, or force a novel workflow.
- IDE chrome and feature density unrelated to Git.
- Interfaces that imitate a terminal at the expense of discoverability or accessibility.

## Design Principles

1. Preserve muscle memory. IDEA-compatible placement, naming, shortcuts, focus behavior, and feedback outrank novelty.
2. Keep Git state visible. Show what will change, what is selected, what is running, and how to recover before asking for trust.
3. Spend resources on the current task. Stream and virtualize large histories, load details on demand, and keep background work cancellable.
4. Use native familiarity. Respect macOS windowing, menus, keyboard conventions, system appearance, reduced motion, and external tools.
5. Make dangerous operations deliberate. Preview scope, use specific action labels, and preserve recovery paths without adding routine friction.

## Accessibility & Inclusion

Target WCAG 2.2 AA for the WebView surface. All core workflows must be keyboard-operable and VoiceOver-labelled. Do not encode graph lanes, file state, conflict state, or operation outcome by color alone. Respect increased contrast, reduced transparency, and reduced motion. Maintain at least 4.5:1 contrast for normal text and clear focus indicators in both light and dark appearances.
