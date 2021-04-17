extern crate byteorder;
extern crate serde_json;

use std::io::Cursor;
use std::io::{Read, Write};
use std::mem;
use std::os::unix::net::UnixStream;

use crate::command::*;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

pub fn send_msg(mut stream: &UnixStream, msg_type: u32, payload: &str) {
    let payload_length = payload.len() as u32;

    let mut msg_prefix: [u8; 6 * mem::size_of::<u8>() + 2 * mem::size_of::<u32>()] =
        *b"i3-ipc00000000";

    msg_prefix[6..]
        .as_mut()
        .write_u32::<LittleEndian>(payload_length)
        .expect("Unable to write");

    msg_prefix[10..]
        .as_mut()
        .write_u32::<LittleEndian>(msg_type)
        .expect("Unable to write");

    let mut msg: Vec<u8> = msg_prefix[..].to_vec();
    msg.extend(payload.as_bytes());

    match stream.write_all(&msg[..]) {
        Err(_) => panic!("couldn't send message"),
        Ok(_) => {}
    }
}

pub fn read_msg(mut stream: &UnixStream) -> Result<String, &str> {
    let mut response_header: [u8; 14] = *b"uninitialized.";
    stream.read_exact(&mut response_header).unwrap();

    if &response_header[0..6] == b"i3-ipc" {
        // let l: [u8; 14] = response_header[6..10];
        let mut v = Cursor::new(vec![
            response_header[6],
            response_header[7],
            response_header[8],
            response_header[9],
        ]);
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
        let mut v = vec![];
        stream.read_to_end(&mut v).unwrap();
        println!("{:?}", v);
        Err("Unable to read i3-ipc packet")
    }
}

pub fn check_success(stream: &UnixStream) {
    match read_msg(&stream) {
        Ok(msg) => {
            let r: Vec<serde_json::Value> = serde_json::from_str(&msg).unwrap();
            match r[0]["success"] {
                serde_json::Value::Bool(true) => println!("Command successful"),
                _ => panic!("Command failed: {:#?}", r),
            }
        }
        Err(_) => panic!("Unable to read response"),
    };
}

pub fn get_outputs(stream: &UnixStream) -> Vec<serde_json::Value> {
    send_msg(&stream, GET_OUTPUTS, "");
    let o = match read_msg(&stream) {
        Ok(msg) => msg,
        Err(_) => panic!("Unable to get outputs"),
    };
    serde_json::from_str(&o).unwrap()
}

pub fn get_workspaces(stream: &UnixStream) -> Vec<serde_json::Value> {
    send_msg(&stream, GET_WORKSPACES, "");
    let ws = match read_msg(&stream) {
        Ok(msg) => msg,
        Err(_) => panic!("Unable to get current workspace"),
    };
    serde_json::from_str(&ws).unwrap()
}

pub fn get_current_output_index(stream: &UnixStream) -> usize {
    let outputs = get_outputs(&stream);
    match outputs
        .iter()
        .position(|x| x["focused"] == serde_json::Value::Bool(true))
    {
        Some(i) => i,
        None => panic!("WTF! No focused output???"),
    }
}

pub fn get_current_output_name(stream: &UnixStream) -> String {
    let outputs = get_outputs(&stream);

    let focused_output_index = match outputs
        .iter()
        .find(|x| x["focused"] == serde_json::Value::Bool(true))
    {
        Some(i) => i["name"].as_str().unwrap(),
        None => panic!("WTF! No focused output???"),
    };

    format!("{}", focused_output_index)
}

pub fn fmt_output_workspace(output: &str, workspace: &str) -> String {
    format!("{}-{}", output, workspace)
}

pub fn get_workspace_by_position(stream: &UnixStream, workspace_pos: &String) -> String {
    let output = get_current_output_name(stream);
    let output_idx = get_current_output_index(stream);
    let workspaces = get_workspaces(&stream);

    let target_workspace = workspaces
        .iter()
        .filter(|x| {
            println!("{}", x["output"]);
            x["output"] == output
        })
        .enumerate()
        .filter(|(i, _)| &(i + 1).to_string() == workspace_pos)
        .next();

    match target_workspace {
        Some((_, w)) => w["name"].as_str().unwrap().to_string(),
        _ => fmt_output_workspace(&output_idx.to_string(), &workspace_pos),
    }
}
