# M2: Extended Functionality

## Goal
Extend the tool for advanced scenarios: workspaces support, alternative package
managers, watch mode, configuration, lifecycle scripts, and diagnostic commands.

## Outcome
After completion, the user can work with monorepo structures (Yarn/Pnpm
workspaces), automatically track changes (watch), configure behavior via
.kleyrc, use lifecycle scripts, and run diagnostic commands (check, list,
clean).

### Progress: 4/27
<progress value="4" max="27"></progress>


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
        "Enhance kley.lock with version pinning"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-15' }
        "Implement clean command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-13' }
        "Add --global flag to link command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-9' }
        "Implement list command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-11' }
        "Implement locations command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-29' }
        "Add progress indicators"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-30' }
        "Consumer lifecycle scripts (prekley/postkley)"<br/><br/>@{ assigned: 🧩, priority: 'Medium', ticket: 'f-37' }
        "kleyrc config"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-24' }
        "Check command"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-25' }
        "Support multiple versions in registry"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-31' }
        "Batch PM installation in install_all"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-34' }
        "Skip PM when deps already satisfied"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-35' }
        "Implement Trusted Publishing"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-36' }
        "Add --workspace/-W flag to add"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-40' }
        "Add --link flag to add (manual link: injection)"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-41' }
        "kley install defaults to link: protocol"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-42' }
        "Add --sig content hash version signature"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-43' }
        "Add --store-folder flag"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-44' }
        "Add --quiet/--no-colors output control"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-45' }
        "Add publish --content preview"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-46' }
        "Add push --replace/--update flags"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-47' }
        "Support publish <sub-project> (monorepo)"<br/><br/>@{ assigned: 🧩, priority: 'Low', ticket: 'f-48' }

    In Progress

    Done
        "Add --pure flag to add (workspaces support)"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-38' }
        "Resolve workspace: protocol in dependencies"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-39' }
        "Publish hooks (.kley/hooks.json + wizard)"<br/><br/>@{ assigned: 🧩, priority: 'High', ticket: 'f-23' }
        "Implement watch command"<br/><br/>@{ assigned: 🧩, priority: 'Very High', ticket: 'f-4' }

```
