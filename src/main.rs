mod util;

use std::env;

use hyprland::data::Monitors;
use hyprland::data::Workspace;
use hyprland::data::Workspaces;
use hyprland::dispatch::*;
use hyprland::shared::HyprData;

use crate::util::*;

fn move_container_to_workspace(workspace_pos: usize) {
    let monitor = Monitors::get().unwrap().find(|it| it.focused).unwrap();
    let workspaces: Vec<Workspace> = Workspaces::get()
        .unwrap()
        .filter(|it| it.monitor == monitor.name)
        .collect();

    let target = if let Some(target) = workspaces.iter().find(|workspace| {
        workspace
            .name
            .starts_with(&format!("{}:{}", workspace.monitor, workspace_pos))
    }) {
        target.name.clone()
    } else {
        // new workspace
        fmt_output_workspace(&monitor.name, &workspace_pos.to_string())
    };

    Dispatch::call(DispatchType::MoveToWorkspaceSilent(
        WorkspaceIdentifier::Name(&target),
        None,
    ))
    .unwrap();
}

fn focus_to_workspace(workspace_pos: usize) {
    let monitor = Monitors::get().unwrap().find(|it| it.focused).unwrap();
    let workspaces: Vec<Workspace> = Workspaces::get()
        .unwrap()
        .filter(|it| it.monitor == monitor.name)
        .collect();

    let target_workspace = workspaces.iter().find(|workspace| {
        workspace
            .name
            .starts_with(&format!("{}:{}", workspace.monitor, workspace_pos))
    });

    if let Some(target_workspace) = target_workspace {
        // pos is active, focus to previous
        if monitor.active_workspace.id == target_workspace.id {
            return;
            /*
            Dispatch::call(DispatchType::Workspace(
                WorkspaceIdentifierWithSpecial::Previous,
            ))
            .unwrap();
            */
        }
    }

    let target = if let Some(target) = target_workspace {
        target.name.clone()
    } else {
        // new workspace
        fmt_output_workspace(&monitor.name, &workspace_pos.to_string())
    };

    Dispatch::call(DispatchType::Workspace(
        WorkspaceIdentifierWithSpecial::Name(&target),
    ))
    .unwrap();
}

fn nomalize_workspace_name() {
    let monitor = Monitors::get().unwrap().find(|it| it.focused).unwrap();
    let workspaces: Vec<Workspace> = Workspaces::get()
        .unwrap()
        .filter(|it| it.monitor == monitor.name)
        .collect();

    workspaces.iter().enumerate().for_each(|(idx, workspace)| {
        let name = fmt_output_workspace(&workspace.monitor, &format!("{}", idx + 1));
        let _ = Dispatch::call(DispatchType::RenameWorkspace(workspace.id, Some(&name)));
    });
}

fn main() {
    // `args` returns the arguments passed to the program
    let args: Vec<String> = env::args().map(|x| x.to_string()).collect();

    nomalize_workspace_name();
    match args[1].as_str() {
        //"init" => init_workspaces(&stream, &args[2]),
        "move" => move_container_to_workspace(args[2].parse().unwrap()),
        "focus" => focus_to_workspace(args[2].parse().unwrap()),
        //"normalize_workspaces_name" => normalize_workspace_name(&stream),
        _ => {}
    }
}
