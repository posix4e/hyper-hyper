#![crate_type = "lib"]
#![crate_name = "hyperhyper"]
#![feature(std_misc)]

extern crate mio;

use mio::*;
use mio::tcp::*;
use std::str::FromStr;
use std::net::SocketAddr;
use std::time::Duration;
use mio::buf::{ByteBuf, MutByteBuf, SliceBuf};
use std::net::Shutdown;
use std::collections::LinkedList;
// Define a handler to process the events
use std::{io, thread};


const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);


struct Echo{
    non_block_client: NonBlock<TcpStream>,
    token: Option<Token>,
    mut_buf: Option<MutByteBuf>,

    buf: Option<ByteBuf>,
    interest: Interest
}

impl Echo {
    fn new(client: NonBlock<TcpStream>) -> Echo {
       Echo {
          non_block_client: client,
          mut_buf: Some(ByteBuf::mut_with_capacity(2048)),
          interest: Interest::hup(),
          buf: None,
          token: None,
       }
   }
}
impl Handler for Echo {
    type Timeout = usize;
    type Message = String;

    fn readable(&mut self, event_loop: &mut EventLoop<Echo>, token: Token, hint: ReadHint) {
        let mut buf = ByteBuf::mut_with_capacity(2048);
        match self.non_block_client.read(&mut buf) {
            Ok(None) => {
                println!("We just got readable, but were unable to read from the socket?");
            }
            Ok(Some(r)) => {
                println!("CONN : we read {} bytes!", r);
                self.interest.remove(Interest::readable());
                self.interest.insert(Interest::writable());
            }
            Err(e) => {
                println!("not implemented; client err={:?}", e);
                self.interest.remove(Interest::readable());
            }
        }
        event_loop.reregister(&self.non_block_client, token, self.interest, PollOpt::edge() | PollOpt::oneshot());

    }

    fn writable(&mut self, event_loop: &mut EventLoop<Echo>, token: Token)  {
        let mut buf = ByteBuf::from_slice("GET /\n".as_bytes());

        match self.non_block_client.write(&mut buf) {
            Ok(None) => {
                println!("client flushing buf; WOULDBLOCK");

                self.buf = Some(buf);
                self.interest.insert(Interest::writable());
            }
            Ok(Some(r)) => {
                println!("CONN : we wrote {} bytes!", r);

                self.mut_buf = Some(buf.flip());

                self.interest.insert(Interest::readable());
                self.interest.remove(Interest::writable());
            }
            Err(e) => println!("not implemented; client err={:?}", e),
        }
        event_loop.reregister(&self.non_block_client, token, self.interest, PollOpt::edge() | PollOpt::oneshot());

    }
    fn notify(&mut self, event_loop: &mut EventLoop<Echo>, msg: String) {
        println!("test3");
    }
}


pub fn google() -> SocketAddr {
    let s = format!("216.58.192.4:{}", 80);
    FromStr::from_str(&s).unwrap()
}


fn get_web_page(hostname: String, port: u32, get_resource: String) {
    let mut event_loop = EventLoop::new().unwrap();
    // == Create & setup client socket

    let url = String::new();

    // == Run test
    println!("Connecting");
    let (mut sock, _) = tcp::v4().unwrap().connect(&google()).unwrap();
    event_loop.register_opt(&sock, CLIENT, Interest::writable(), PollOpt::edge() | PollOpt::oneshot()).unwrap();
    event_loop.run(&mut Echo::new(sock));
}

#[test]
 fn test(){
    get_web_page("www.google.com".to_string(), 80, "/".to_string());

}
