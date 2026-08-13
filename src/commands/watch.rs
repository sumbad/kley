use anyhow::Result;
use colored::*;
use notify::event::ModifyKind;
use notify::{Event, EventKind, RecursiveMode, Watcher, recommended_watcher};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use crate::commands::publish::publish;
use crate::emoji;
use crate::package::Package;
use crate::registry::Registry;

const DEBOUNCE: Duration = Duration::from_millis(500);

pub fn watch(registry: &mut Registry, watch_path: &Option<String>) -> Result<()> {
    let package = Package::get(&std::env::current_dir()?)?;
    let name = package.json.name.clone();
    let version = package.json.version.clone();
    let watch_path = if let Some(path) = watch_path {
        Path::new(path)
    } else {
        Path::new(".")
    };

    println!(
        "{} Watching {}@{} for changes...",
        emoji::WAITING,
        name.cyan(),
        version.magenta()
    );
    println!("{}", "  Press Ctrl+C to stop.".bright_black().italic());

    let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
    let mut watcher = recommended_watcher(tx)?;
    watcher.watch(watch_path, RecursiveMode::Recursive)?;

    let mut pending = false;

    loop {
        match rx.recv_timeout(DEBOUNCE) {
            Ok(Ok(event)) => {
                if is_relevant_event(&event) {
                    pending = true;
                }
            }
            Ok(Err(e)) => {
                eprintln!("{} Watch error: {}", emoji::ERROR, e);
            }
            Err(RecvTimeoutError::Timeout) => {
                if pending {
                    pending = false;
                    println!(
                        "\n{} Changes detected, republishing {}...",
                        emoji::PUBLISH,
                        name.cyan()
                    );

                    if let Err(e) = publish(registry, true, true, false, false) {
                        eprintln!("{} Publish error: {}", emoji::ERROR, e);
                    }

                    println!(
                        "{} Watching for changes... (Ctrl+C to stop)",
                        emoji::WAITING
                    );
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    Ok(())
}

fn is_relevant_event(event: &Event) -> bool {
    let relevant_kind = matches!(
        event.kind,
        EventKind::Create(_)
            | EventKind::Remove(_)
            | EventKind::Modify(ModifyKind::Data(_) | ModifyKind::Any | ModifyKind::Name(_))
    );

    if !relevant_kind {
        return false;
    }

    event.paths.iter().any(|p| {
        !p.components().any(|c| {
            let s = c.as_os_str().to_string_lossy();
            s == "node_modules" || s == ".git" || s == ".kley"
        })
    })
}
