// client.rs
extern crate byteorder;

use std::env;
use std::path::Path;
use std::io::{Read, Write};
use std::process::exit;
use std::os::unix::net::UnixStream;
use std::mem;

use byteorder::{WriteBytesExt, LittleEndian};


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

    // let magic = b"i3-ipc";

    // let mut msg = b"i3-ipc".to_vec();
    let mut msg: [u8; 6 * mem::size_of::<u8>() + 2 * mem::size_of::<u32>()] = *b"i3-ipc00000000";

    msg[6..].as_mut()
        .write_u32::<LittleEndian>(payload_length)
        .expect("Unable to write");

    msg[10..].as_mut()
        .write_u32::<LittleEndian>(msg_type)
        .expect("Unable to write");

    // let msg = format!("{}{}{}{}", magic, pl_t, pl_l, payload);
    println!("msg: {:x?}", msg);

    match stream.write_all(&msg) {
        Err(_) => panic!("couldn't send message"),
        Ok(_) => {}
    }
}


fn main() {
    // `args` returns the arguments passed to the program
    let args: Vec<String> = env::args().map(|x| x.to_string())
                                       .collect();

    let mut stream = get_stream();

    // stream.set_nonblocking(true).expect("could not set non blocking");

    send_msg(&stream, 1, "");

    let mut response_header: [u8; 14] = *b"..............";
    // let mut response = vec!();
    stream.read_exact(&mut response_header);
    // stream.read_to_string(&mut response);
    println!("answer: {:x?}", response_header);
}
