// client.rs
extern crate byteorder;
extern crate serde_json;

use std::env;
use std::path::Path;
use std::io::{Read, Write};
use std::process::exit;
use std::os::unix::net::UnixStream;
use std::mem;
use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

const RUN_COMMAND: u32 = 0;
const GET_WORKSPACES: u32 = 1;
const SUBSCRIBE: u32 = 2;
const GET_OUTPUTS: u32 = 3;



fn get_stream() -> UnixStream {
    let socket_path = match env::var("I3SOCK") {
        Ok(val) => val,
        Err(_e) => {
            println!("couldn't find i3/sway socket");
            exit(1);
        },
    };

    let socket = Path::new(&socket_path);

    // Connect to socket
    match UnixStream::connect(&socket) {
        Err(_) => panic!("couldn't connect to i3/sway socket"),
        Ok(stream) => stream,
    }
}


fn send_msg(mut stream: &UnixStream, msg_type: u32, payload: &str) {
    let payload_length = payload.len() as u32;

    let mut msg_prefix: [u8; 6 * mem::size_of::<u8>() + 2 * mem::size_of::<u32>()] = *b"i3-ipc00000000";

    msg_prefix[6..].as_mut()
        .write_u32::<LittleEndian>(payload_length)
        .expect("Unable to write");

    msg_prefix[10..].as_mut()
        .write_u32::<LittleEndian>(msg_type)
        .expect("Unable to write");

    let mut msg: Vec<u8> = msg_prefix[..].to_vec();
    msg.extend(payload.as_bytes());

    match stream.write_all(&msg[..]) {
        Err(_) => panic!("couldn't send message"),
        Ok(_) => {}
    }
}


fn read_msg(mut stream: &UnixStream) -> Result<String, &str> {
    let mut response_header: [u8; 14] = *b"uninitialized.";
    stream.read_exact(&mut response_header).unwrap();

    if &response_header[0..6] == b"i3-ipc" {
        // let l: [u8; 14] = response_header[6..10];
        let mut v = Cursor::new(vec!(response_header[6], response_header[7], response_header[8], response_header[9]));
        // let mut v = Cursor::new(vec!(response_header[6..10]));
        let payload_length = v.read_u32::<LittleEndian>().unwrap();
        // payload_length = response_header[6..10].read_u32::<LittleEndian>().unwrap();
        println!("This is a valid i3 packet of length: {}", payload_length);

        let mut payload = vec![0; payload_length as usize];
        stream.read_exact(&mut payload[..]).unwrap();
        let payload_str = String::from_utf8(payload).unwrap();
        println!("Payload: {}", payload_str);
        Ok(payload_str)
    } else {
        print!("Not an i3-icp packet, emptying the buffer: ");
        let mut v = vec!();
        stream.read_to_end(&mut v).unwrap();
        println!("{:?}", v);
        Err("Unable to read i3-ipc packet")
    }
}

fn check_success(stream: &UnixStream) {
    match read_msg(&stream) {
        Ok(msg) => {
            let r: Vec<serde_json::Value> = serde_json::from_str(&msg).unwrap();
            match r[0]["success"] {
                serde_json::Value::Bool(true) => println!("Command successful"),
                _ => panic!("Command failed: {:#?}", r),
            }
        },
        Err(_) => panic!("Unable to read response"),
    };
}

fn get_outputs(stream: &UnixStream) -> Vec<serde_json::Value> {
    send_msg(&stream, GET_OUTPUTS, "");
    let o = match read_msg(&stream) {
        Ok(msg) => msg,
        Err(_) => panic!("Unable to get outputs"),
    };
    serde_json::from_str(&o).unwrap()
}

fn get_workspaces(stream: &UnixStream) -> Vec<serde_json::Value> {
    send_msg(&stream, GET_WORKSPACES, "");
    let ws = match read_msg(&stream) {
        Ok(msg) => msg,
        Err(_) => panic!("Unable to get current workspace"),
    };
    serde_json::from_str(&ws).unwrap()
}

fn get_current_output_name(stream: &UnixStream) -> String {
    let outputs = get_outputs(&stream);

    let focused_output_index = match outputs.iter().position(|x| x["focused"] == serde_json::Value::Bool(true)) {
        Some(i) => i,
        None => panic!("WTF! No focused output???"),
    };

    format!("{}", focused_output_index)
}

fn move_container_to_workspace(stream: &UnixStream, workspace_name: &String) {
    let mut cmd: String = "move container to workspace ".to_string();
    let output = get_current_output_name(stream);
    cmd.push_str(&output);
    cmd.push_str(&workspace_name);
    println!("Sending command: '{}'", &cmd);
    send_msg(&stream, RUN_COMMAND, &cmd);
    check_success(&stream);
}

fn focus_to_workspace(stream: &UnixStream, workspace_name: &String) {
    let mut cmd: String = "workspace ".to_string();
    let output = get_current_output_name(stream);
    cmd.push_str(&output);
    cmd.push_str(&workspace_name);
    println!("Sending command: '{}'", &cmd);
    send_msg(&stream, RUN_COMMAND, &cmd);
    check_success(&stream);
}

fn move_container_to_next_output(stream: &UnixStream) {
    move_container_to_next_or_prev_output(&stream, false);
}

fn move_container_to_prev_output(stream: &UnixStream) {
    move_container_to_next_or_prev_output(&stream, true);
}

fn move_container_to_next_or_prev_output(stream: &UnixStream, go_to_prev: bool) {
    let outputs = get_outputs(&stream);
    let focused_output_index = match outputs.iter().position(|x| x["focused"] == serde_json::Value::Bool(true)) {
        Some(i) => i,
        None => panic!("WTF! No focused output???"),
    };

    let target_output;
    if go_to_prev {
        target_output = &outputs[(focused_output_index - 1 + &outputs.len()) % &outputs.len()];
    } else {
        target_output = &outputs[(focused_output_index + 1) % &outputs.len()];
    }

    let workspaces = get_workspaces(&stream);
    let target_workspace = workspaces.iter()
                            .filter(|x| x["output"] == target_output["name"] && x["visible"] == serde_json::Value::Bool(true))
                            .next().unwrap();

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

fn init_workspaces(stream: &UnixStream) {
    let outputs = get_outputs(&stream);

    let cmd_prefix: String = "focus output ".to_string();
    for output in outputs.iter().rev() {
        let mut cmd = cmd_prefix.clone();
        cmd.push_str(&output["name"].as_str().unwrap());
        println!("Sending command: '{}'", &cmd);
        send_msg(&stream, RUN_COMMAND, &cmd);
        check_success(&stream);
        focus_to_workspace(&stream, &"1".to_string());
    }
}

fn main() {
    // `args` returns the arguments passed to the program
    let args: Vec<String> = env::args().map(|x| x.to_string())
                                       .collect();

    let stream = get_stream();

    match args[1].as_str() {
        "init" => init_workspaces(&stream),
        "move" => move_container_to_workspace(&stream, &args[2]),
        "focus" => focus_to_workspace(&stream, &args[2]),
        "next_output" => move_container_to_next_output(&stream),
        "prev_output" => move_container_to_prev_output(&stream),
        _ => {},
    }
}
