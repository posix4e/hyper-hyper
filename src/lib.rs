#![crate_type = "lib"]
#![crate_name = "hyperhyper"]
#![feature(lookup_host)]
#![feature(ip_addr)]
#![feature(collections)]
extern crate mio;

use mio::*;
use mio::tcp::*;
use std::str::FromStr;
use std::net::SocketAddr;
use mio::buf::{ByteBuf, MutByteBuf};
use std::rc::Rc;

// Define a handler to process the events
const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);

enum HTTP_ACTION {
    Get(Rc<String>),
}

struct Echo {
    non_block_client: TcpStream,
    action: HTTP_ACTION,
    token: Option<Token>,
    mut_buf: Option<MutByteBuf>,
    buf: Option<ByteBuf>,
    interest: Interest
}

impl Echo {
    fn new(client: TcpStream, action: HTTP_ACTION) -> Echo {
        Echo {
            non_block_client: client,
            action: action,
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
                if (r > 0) {
                    event_loop.reregister(&self.non_block_client, token, self.interest,
                                          PollOpt::edge() | PollOpt::oneshot());
                } else {
                    event_loop.shutdown();
                }
            }
            Err(e) => {
                println!("not implemented; client err={:?}", e);
                self.interest.remove(Interest::readable());
            }
        }
    }

    fn writable(&mut self, event_loop: &mut EventLoop<Echo>, token: Token) {
       match (self.action) {
       		HTTP_ACTION::Get(ref resource) => {
       				let get_command: String = String::from_str("GET ") + resource;
					let mut buf = ByteBuf::from_slice(get_command.as_bytes());
					println!("GET {}",resource);
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
                        event_loop.reregister(&self.non_block_client, token, self.interest,
                                              PollOpt::edge() | PollOpt::oneshot());

			
       			}
       }
                   //_ => {}
            
    }
    fn notify(&mut self, event_loop: &mut EventLoop<Echo>, msg: String) {
        println!("test3");
    }
}

pub fn google() -> SocketAddr {
    let s = format!("216.58.192.4:{}", 80);
    FromStr::from_str(&s).unwrap()
}

fn get_web_page(hostname: String, port: u16, action: HTTP_ACTION) {
    let mut event_loop = EventLoop::new().unwrap();
    // == Create & setup client socket

    let url = String::new();

    // == Run test
    println!("Connecting");
    let ip = std::net::lookup_host(&hostname).unwrap().next().unwrap().unwrap();
    let address = SocketAddr::new(ip.ip(), port);
    let (sock, _) = TcpSocket::v4().unwrap().connect(&address).unwrap();
    event_loop.register_opt(&sock, CLIENT, Interest::writable(),
                            PollOpt::edge() | PollOpt::oneshot()).unwrap();
    event_loop.run(&mut Echo::new(sock, action));
}

#[test]
fn test() {
    println!("test");
    let path = String::from_str("/");
    let resource = Rc::new(path);
    get_web_page("www.google.com".to_string(), 80, HTTP_ACTION::Get(resource));
}
