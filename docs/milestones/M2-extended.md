# M2: Extended Functionality

## Goal
Extend the tool for advanced scenarios: workspaces support, alternative package
managers, watch mode, configuration, lifecycle scripts, and diagnostic commands.

## Outcome
After completion, the user can work with monorepo structures (Yarn/Pnpm
workspaces), automatically track changes (watch), configure behavior via
.kleyrc, use lifecycle scripts, and run diagnostic commands (check, list,
clean).

### Progress: 0/15
<progress value="0" max="15"></progress>


```mermaid
---
config:
  kanban:
    ticketBaseUrl: 'https://github.com/sumbad/kley/tree/master/docs/tickets/#TICKET#.md'
    
    ### Legend
    # assigned - 🧩 = feat | 🐞 = bug
    # priority - `Very High` = Critical | `High` = High | `Low` = Medium | `Very Low` = Low
    # ticket - used for a ticket file name (tickets/#TICKET#.md)
---
%%{init: {
  'theme': 'base',
  'themeVariables': {
    'darkMode': false,
    'background': '#F0F2EB'
  }
}}%%

kanban
    Todo
        "Implement retreat and restore commands"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-21' }
        "Add --changed flag to publish command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-22' }
        "Implement watch command"<br/><br/>@{ assigned: 🧩, priority: 'Very High', ticket: 'f-4' }
        "Enhance kley.lock with version pinning"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-15' }
        "Implement clean command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-13' }
        "Add --global flag to link command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-9' }
        "Implement list command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-11' }
        "Implement locations command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-29' }
        "Add progress indicators"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-30' }
        "Lifecycle scripts"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-23' }
        "kleyrc config"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-24' }
        "Check command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-25' }
        "Support multiple versions in registry"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-31' }
        "Batch PM installation in install_all"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-34' }
        "Skip PM when deps already satisfied"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-35' }

    In Progress

    Done

```
