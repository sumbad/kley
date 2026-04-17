# Project Board

This board tracks the progress of development tasks for the kley project.

**Epics:**
- **I:** Core Publishing & Adding
- **II:** Publish Automation & Linking Speed
- **III:** Streamlined Local Package Workflow
- **IV:** Yarn/Pnpm Workspaces Support
- **V:** Monorepos & Sub-projects
- **VI:** DX/UX Improvements (General)

**Complexity Estimate (color):**
- `Very High`: Complex task, may require significant refactoring or research.
- `High`: A feature with multiple components.
- `Low`: A small, well-defined task.
- `Very Low`: A trivial change.

**NOTE:**
- When setting the `priority` field in the Mermaid Kanban diagram, use only the following allowed values: `'Very High'`, `'High'`, `'Low'`, `'Very Low'`.
- Kanban task titles should not contain special characters like ()@".
- Use only alphanumeric characters, spaces, and hyphens.
---

```mermaid
---
config:
  kanban:
    ticketBaseUrl: 'https://github.com/sumbad/kley/tree/master/docs/tickets/#TICKET#.md'
    
    ### Legend
    # assigned - used for epics, has an epic name (epics/#EPIC#.md)
    # priority - used for complexity
    # ticket - used for a ticket file name (tickets/#TICKET#.md)
---

kanban
    Backlog
        "Enhance `kley.lock` with version pinning"<br/><br/>@{ assigned: VI, priority: 'High', ticket: '016-enhance-kley-lock' }
        "Implement `clean` command"<br/><br/>@{ assigned: VI, priority: 'Low', ticket: '014-clean-command' }
        "Implement watch command"<br/><br/>@{ assigned: VI, priority: 'Very High', ticket: '004-watch-command' }
        "Add --global flag to link command"<br/><br/>@{ assigned: VI, priority: 'Low', ticket: '010-link-global-flag' }
        "Implement list command"<br/><br/>@{ assigned: VI, priority: 'Low', ticket: '012-list-command' }
        "Implement locations command"<br/><br/>@{ assigned: VI, priority: 'Low' }
        "Add progress indicators"<br/><br/>@{ assigned: VI, priority: 'Low' }

    [Todo]
        "Implement install Command"<br/><br/>@{ assigned: III, priority: 'High', ticket: '017-implement-install-command' }
        "Implement install no args as Update All"<br/><br/>@{ assigned: III, priority: 'High', ticket: '020-install-no-args-as-update' }
        "Future: Implement Fast Install Optimization"<br/><br/>@{ assigned: III, priority: 'High', ticket: '019-fast-install-optimization' }

    [In progress]
        "Implement Robust Package Manager Detection"<br/><br/>@{ assigned: III, priority: 'High', ticket: '018-robust-pm-detection' }

    Done
        "Fix `add` command to support version syntax"<br/><br/>@{ assigned: II, priority: 'Very Low' }
        "More information for CLI description"<br/><br/>@{ assigned: VI, priority: 'Low' }
        "Implement `unpublish` command"<br/><br/>@{ assigned: II, priority: 'High', ticket: '013-unpublish-command' }
        "Implement `update` command"<br/><br/>@{ assigned: II, priority: 'High', ticket: '011-update-command' }
        "Add `--push` flag to `publish` command"<br/><br/>@{ assigned: II, priority: 'Very High', ticket: '001-push-command' }
        "Implement Global Package Registry `registry.json`"<br/><br/>@{ assigned: II, priority: 'High', ticket: '015-installation-registry' }
        "Implement `link` command"<br/><br/>@{ assigned: II, priority: 'High', ticket: '009-link-command' }
        "Add base integration tests"<br/><br/>@{ assigned: II, priority: 'Low' }
        "Fix the publish command to simulate npm logic"<br/><br/>@{ assigned: II, priority: 'Low' }
        "Publish to crates.io and npm"<br/><br/>@{ assigned: II, priority: 'Low' }
        "Preserve package.json order on add"<br/><br/>@{ assigned: VI, priority: 'High', ticket: '008-preserve-pkg-order' }
        "Implement remove command"<br/><br/>@{ assigned: I, priority: 'High', ticket: '007-remove-command' }
        "Create and manage kley.lock"<br/><br/>@{ assigned: I, priority: 'High', ticket: '006-kley-lock' }
        "Use ignore crate for filtering"<br/><br/>@{ assigned: I, priority: 'High', ticket: '002-ignore-crate' }
        "Refactor main.rs to command modules"<br/><br/>@{ assigned: VI, priority: 'High', ticket: '005-refactor-main' }
        "Automate package.json modification"<br/><br/>@{ assigned: I, priority: 'High', ticket: '003-auto-pkg-json' }
        "Add unit tests for package.json logic"<br/><br/>@{ assigned: VI, priority: 'High' }
        "Improve Documentation with Mermaid"<br/><br/>@{ assigned: VI, priority: 'Low' }
        "Dependency audit home to dirs"<br/><br/>@{ assigned: I, priority: 'Low' }
```

