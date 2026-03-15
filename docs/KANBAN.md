# Project Board

This board tracks the progress of development tasks for the kley project.

**Epics:**
- **I:** Core Publishing & Adding
- **II:** Publish Automation & Linking Speed
- **III:** Yarn/Pnpm Workspaces Support
- **IV:** Monorepos & Sub-projects
- **V:** DX/UX Improvements

**Complexity Estimate (color):**
- `Very High`: Complex task, may require significant refactoring or research.
- `High`: A feature with multiple components.
- `Low`: A small, well-defined task.
- `Very Low`: A trivial change.


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
        "Implement list command"<br/><br/>@{ assigned: V, priority: 'Very Low' }
        "Implement locations command"<br/><br/>@{ assigned: V, priority: 'Low' }
        "Add progress indicators"<br/><br/>@{ assigned: V, priority: 'Low' }
        "Add comprehensive tests"<br/><br/>@{ priority: 'Very High' }

    [Todo]
        "Add base integration tests"<br/><br/>@{ assigned: II, priority: 'Low' }
        "Publish to crates.io and npm"<br/><br/>@{ assigned: II, priority: 'Low' }
        "Implement push command"<br/><br/>@{ assigned: II, priority: 'Very High', ticket: '001-push-command' }
        "Implement watch command"<br/><br/>@{ assigned: II, priority: 'Very High', ticket: '004-watch-command' }

    [In progress]
        "Fix the publish command to simulate npm logic"<br/><br/>@{ assigned: II, priority: 'Low' }

    Done
        "Implement remove command"<br/><br/>@{ assigned: I, priority: 'High', ticket: '007-remove-command' }
        "Create and manage kley.lock"<br/><br/>@{ assigned: I, priority: 'High', ticket: '006-kley-lock' }
        "Use ignore crate for filtering"<br/><br/>@{ assigned: I, priority: 'High', ticket: '002-ignore-crate' }
        "Refactor main.rs to command modules"<br/><br/>@{ assigned: V, priority: 'Medium', ticket: '005-refactor-main' }
        "Automate package.json modification"<br/><br/>@{ assigned: I, priority: 'High', ticket: '003-auto-pkg-json' }
        "Add unit tests for package.json logic"<br/><br/>@{ assigned: V, priority: 'Medium' }
        "Improve Documentation with Mermaid"<br/><br/>@{ assigned: V, priority: 'Low' }
        "Dependency audit home to dirs"<br/><br/>@{ assigned: I, priority: 'Low' }
```

