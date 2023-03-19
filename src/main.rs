mod command;
mod util;

// client.rs
extern crate serde_json;

use std::env;
use std::os::unix::net::UnixStream;

use hyprland::data::Monitors;
use hyprland::data::Workspace;
use hyprland::data::Workspaces;
use hyprland::dispatch::*;
use hyprland::shared::HyprData;

use crate::command::*;
use crate::util::*;

fn move_container_to_workspace(workspace_pos: usize) {
    let monitor = Monitors::get().unwrap().find(|it| it.focused).unwrap();
    let workspaces: Vec<Workspace> = Workspaces::get()
        .unwrap()
        .filter(|it| it.monitor == monitor.name)
        .collect();

    let target = if workspace_pos <= workspaces.len() {
        workspaces.get(workspace_pos - 1).unwrap().name.clone()
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

    let target = if workspace_pos <= workspaces.len() {
        workspaces.get(workspace_pos - 1).unwrap().name.clone()
    } else {
        // new workspace
        fmt_output_workspace(&monitor.name, &workspace_pos.to_string())
    };

    Dispatch::call(DispatchType::Workspace(
        WorkspaceIdentifierWithSpecial::Name(&target),
    ))
    .unwrap();
}

fn init_workspace(stream: &UnixStream, output: String, workspace: &String) {
    let mut cmd: String = "workspace ".to_string();
    cmd.push_str(&fmt_output_workspace(&output, &workspace));
    println!("Sending command: '{}'", &cmd);
    send_msg(&stream, RUN_COMMAND, &cmd);
    check_success(&stream);
}

fn focus_all_outputs_to_workspace(stream: &UnixStream, workspace_name: &String) {
    let current_output = get_current_output_name(stream);
    println!("Current output name: {}", current_output);

    // Iterate on all outputs to focus on the given workspace
    let outputs = get_outputs(&stream);
    for output in outputs.iter() {
        let mut cmd: String = "focus output ".to_string();
        cmd.push_str(&output["name"].as_str().unwrap());
        println!("Sending command: '{}'", &cmd);
        send_msg(&stream, RUN_COMMAND, &cmd);
        check_success(&stream);

        //focus_to_workspace(&stream, &workspace_name);
    }

    // Get back to currently focused output
    let mut cmd: String = "focus output ".to_string();
    cmd.push_str(&current_output);
    println!("Sending command: '{}'", &cmd);
    send_msg(&stream, RUN_COMMAND, &cmd);
    check_success(&stream);
}

fn normalize_workspace_name(stream: &UnixStream) {
    let workspaces = get_workspaces(stream);
    let outputs = get_outputs(stream);
    let mut rename_to_temp: Vec<String> = Vec::new();
    let mut rename_from_temp: Vec<String> = Vec::new();

    outputs.iter().enumerate().for_each(|(output_idx, output)| {
        workspaces
            .iter()
            .filter(|workspace| {
                workspace["output"].as_str().unwrap() == output["name"].as_str().unwrap()
            })
            .enumerate()
            .for_each(|(workspace_idx, workspace)| {
                let old_name = &workspace["name"].as_str().unwrap();
                let new_name = &fmt_output_workspace(
                    &format!("{}", output_idx),
                    &format!("{}", workspace_idx + 1),
                );
                let cmd_to_temp = format!(
                    "rename workspace \"{}\" to \"temp-{}\" ",
                    old_name, new_name
                );
                let cmd_from_temp = format!(
                    "rename workspace \"temp-{}\" to \"{}\" ",
                    new_name, new_name
                );
                rename_to_temp.push(cmd_to_temp);
                rename_from_temp.push(cmd_from_temp);
            })
    });

    rename_to_temp.iter().for_each(|cmd| {
        send_msg(&stream, RUN_COMMAND, &cmd);
        check_success(&stream);
    });
    rename_from_temp.iter().for_each(|cmd| {
        send_msg(&stream, RUN_COMMAND, &cmd);
        check_success(&stream);
    })
}

fn move_container_to_next_output(stream: &UnixStream) {
    move_container_to_next_or_prev_output(&stream, false);
}

fn move_container_to_prev_output(stream: &UnixStream) {
    move_container_to_next_or_prev_output(&stream, true);
}

fn move_container_to_next_or_prev_output(stream: &UnixStream, go_to_prev: bool) {
    let outputs = get_outputs(&stream);
    let focused_output_index = get_current_output_index(&stream);
    let target_output;
    if go_to_prev {
        target_output = &outputs[(focused_output_index - 1 + &outputs.len()) % &outputs.len()];
    } else {
        target_output = &outputs[(focused_output_index + 1) % &outputs.len()];
    }

    let workspaces = get_workspaces(&stream);
    let target_workspace = workspaces
        .iter()
        .filter(|x| {
            x["output"] == target_output["name"] && x["visible"] == serde_json::Value::Bool(true)
        })
        .next()
        .unwrap();

    // Move container to target workspace
    let mut cmd: String = "move container to workspace ".to_string();
    cmd.push_str(&target_workspace["name"].as_str().unwrap());
    send_msg(&stream, RUN_COMMAND, &cmd);
    check_success(&stream);

    // Focus that workspace to follow the container
    let mut cmd: String = "workspace ".to_string();
    cmd.push_str(&target_workspace["name"].as_str().unwrap());
    send_msg(&stream, RUN_COMMAND, &cmd);
    check_success(&stream);
}

fn init_workspaces(stream: &UnixStream, workspace_name: &String) {
    let cmd_prefix: String = "focus output ".to_string();
    get_outputs(&stream)
        .iter()
        .enumerate()
        .rev()
        .for_each(|(i, output)| {
            let mut cmd = cmd_prefix.clone();
            cmd.push_str(&output["name"].as_str().unwrap());
            println!("Sending command: '{}'", &cmd);
            send_msg(&stream, RUN_COMMAND, &cmd);
            check_success(&stream);
            init_workspace(stream, format!("{}", i), workspace_name);
        });
}

fn main() {
    // `args` returns the arguments passed to the program
    let args: Vec<String> = env::args().map(|x| x.to_string()).collect();

    match args[1].as_str() {
        //"init" => init_workspaces(&stream, &args[2]),
        "move" => move_container_to_workspace(args[2].parse().unwrap()),
        "focus" => focus_to_workspace(args[2].parse().unwrap()),
        //"normalize_workspaces_name" => normalize_workspace_name(&stream),
        _ => {}
    }
}
