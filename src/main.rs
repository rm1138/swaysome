// client.rs
extern crate byteorder;
extern crate serde;
extern crate serde_json;

use std::env;
use std::path::Path;
use std::io::{Read, Write};
use std::process::exit;
use std::os::unix::net::UnixStream;
use std::mem;
use std::io::Cursor;

use serde::{Deserialize, Serialize};

use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};

const RUN_COMMAND: u32 = 0;
const GET_WORKSPACES: u32 = 1;
const SUBSCRIBE: u32 = 2;
const GET_OUTPUTS: u32 = 3;

#[derive(Serialize, Deserialize)]
struct WorkspaceRect {
    x: usize,
    y: usize,
}

#[derive(Serialize, Deserialize)]
struct Workspace {
    num: usize,
    name: String,
    visible: bool,
    focused: bool,
    rect: WorkspaceRect,
    output: String,
}

#[derive(Serialize, Deserialize)]
struct OutputMode {
    width: usize,
    height: usize,
    refresh: usize,
}

#[derive(Serialize, Deserialize)]
struct Output {
    name: String,
    make: String,
    model: String,
    serial: String,
    active: bool,
    primary: bool,
    focused: bool,
    scale: f32,
    subpixel_hinting: String,
    transform: String,
    current_workspace: String,
    modes: Vec<OutputMode>,
    current_mode: OutputMode,
}


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

fn get_current_output_name(stream: &UnixStream) -> String {
    send_msg(&stream, GET_OUTPUTS, "");
    let o = match read_msg(&stream) {
        Ok(msg) => msg,
        Err(_) => panic!("Unable to get current workspace"),
    };
    let outputs: Vec<Output> = serde_json::from_str(&o).unwrap();

    let focused_output_index = match outputs.iter().position(|x| x.focused) {
        Some(i) => i,
        None => panic!("WTF! No focused output???"),
    };

    // outputs[focused_output_index].name.clone()
    format!("{}", focused_output_index)
}

fn get_current_workspace_name(stream: &UnixStream) -> String {
    send_msg(&stream, GET_WORKSPACES, "");
    let ws = match read_msg(&stream) {
        Ok(msg) => msg,
        Err(_) => panic!("Unable to get current workspace"),
    };
    let workspaces: Vec<Workspace> = serde_json::from_str(&ws).unwrap();

    let focused_workspace_index = match workspaces.iter().position(|x| x.focused) {
        Some(i) => i,
        None => panic!("WTF! No focused workspace???"),
    };

    workspaces[focused_workspace_index].name.clone()
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

fn main() {
    // `args` returns the arguments passed to the program
    let args: Vec<String> = env::args().map(|x| x.to_string())
                                       .collect();

    let mut stream = get_stream();

    match args[1].as_str() {
        "move" => move_container_to_workspace(&stream, &args[2]),
        "focus" => focus_to_workspace(&stream, &args[2]),
        _ => {},
    }
}
