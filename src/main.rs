// client.rs
extern crate byteorder;

use std::env;
use std::path::Path;
use std::io::{Read, Write};
use std::process::exit;
use std::os::unix::net::UnixStream;
use std::mem;
use std::io::Cursor;

use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};


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


fn main() {
    // `args` returns the arguments passed to the program
    let args: Vec<String> = env::args().map(|x| x.to_string())
                                       .collect();

    let mut stream = get_stream();

    send_msg(&stream, 1, "");

    read_msg(&stream);

}
