# Project Board

This board tracks the progress of development tasks for the kley project.

**Epics:**
- **I:** Core Publishing & Linking
- **II:** Update Propagation
- **III:** Project Management
- **IV:** DX/UX Improvements
- **V:** Code Quality & Testing

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
        "Implement push command"<br/><br/>@{ assigned: II, priority: 'Very High', ticket: '001-push-command' }
        "Implement remove command"<br/><br/>@{ assigned: III, priority: 'High' }
        "Implement list command"<br/><br/>@{ assigned: III, priority: 'Very Low' }
        "Implement locations command"<br/><br/>@{ assigned: III, priority: 'Low' }
        "Add progress indicators"<br/><br/>@{ assigned: IV, priority: 'Low' }
        "Implement watch command"<br/><br/>@{ assigned: IV, priority: 'Very High', ticket: '004-watch-command' }
        "Add comprehensive tests"<br/><br/>@{ assigned: V, priority: 'Very High' }

    Todo

    [In progress]

    Done
        "Create and manage kley.lock"<br/><br/>@{ assigned: I, priority: 'High', ticket: '006-kley-lock' }
        "Use ignore crate for filtering"<br/><br/>@{ assigned: I, priority: 'High', ticket: '002-ignore-crate' }
        "Refactor main.rs to command modules"<br/><br/>@{ assigned: V, priority: 'Medium', ticket: '005-refactor-main' }
        "Automate package.json modification"<br/><br/>@{ assigned: I, priority: 'High', ticket: '003-auto-pkg-json' }
        "Add unit tests for package.json logic"<br/><br/>@{ assigned: V, priority: 'Medium' }
        "Improve Documentation with Mermaid"<br/><br/>@{ assigned: V, priority: 'Low' }
        "Dependency audit home to dirs"<br/><br/>@{ assigned: I, priority: 'Low' }
```

