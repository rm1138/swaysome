mod command;
mod util;

// client.rs
extern crate serde_json;

use std::env;
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::exit;

use crate::command::*;
use crate::util::*;

fn get_stream() -> UnixStream {
    let socket_path = match env::var("I3SOCK") {
        Ok(val) => val,
        Err(_e) => {
            println!("couldn't find i3/sway socket");
            exit(1);
        }
    };

    let socket = Path::new(&socket_path);

    // Connect to socket
    match UnixStream::connect(&socket) {
        Err(_) => panic!("couldn't connect to i3/sway socket"),
        Ok(stream) => stream,
    }
}

fn move_container_to_workspace(stream: &UnixStream, workspace_pos: &String) {
    let mut cmd: String = "move container to workspace ".to_string();
    let workspace = get_workspace_by_position(stream, workspace_pos);
    cmd.push_str(&workspace);
    send_msg(&stream, RUN_COMMAND, &cmd);
    check_success(&stream);
}

fn focus_to_workspace(stream: &UnixStream, workspace_pos: &String) {
    let mut cmd: String = "workspace ".to_string();
    let workspace = get_workspace_by_position(stream, workspace_pos);
    cmd.push_str(&workspace);
    println!("Sending command: '{}'", &cmd);
    send_msg(&stream, RUN_COMMAND, &cmd);
    check_success(&stream);
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

        focus_to_workspace(&stream, &workspace_name);
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

    let stream = get_stream();

    match args[1].as_str() {
        "init" => init_workspaces(&stream, &args[2]),
        "move" => move_container_to_workspace(&stream, &args[2]),
        "focus" => focus_to_workspace(&stream, &args[2]),
        "focus_all_outputs" => focus_all_outputs_to_workspace(&stream, &args[2]),
        "next_output" => move_container_to_next_output(&stream),
        "prev_output" => move_container_to_prev_output(&stream),
        "normalize_workspaces_name" => normalize_workspace_name(&stream),
        _ => {}
    }
}
