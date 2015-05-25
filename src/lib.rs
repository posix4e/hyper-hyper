#![crate_type = "lib"]
#![crate_name = "hyperhyper"]
#![feature(collections)]
#![feature(lookup_host)]
#![feature(ip_addr)]

extern crate mio;
use mio::*;
use mio::tcp::*;
use std::net::SocketAddr;

use mio::buf::{ByteBuf, MutByteBuf};
use std::rc::Rc;

// Define a handler to process the events
const SERVER: Token = Token(0);
const CLIENT: Token = Token(1);

pub enum HttpAction {
    Get(Rc<String>),
}

pub struct Echo {
    non_block_client: TcpStream,
    action: HttpAction,
    token: Option<Token>,
    mut_buf: Option<MutByteBuf>,
    buf: Option<ByteBuf>,
    interest: Interest
}

impl Echo {
    pub fn new(client: TcpStream, action: HttpAction) -> Echo {
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
        println!("Read");
        let mut buf = ByteBuf::mut_with_capacity(2048);
        match self.non_block_client.read(&mut buf) {
            Ok(None) => {
                println!("We just got readable, but were unable to read from the socket?");
            }
            Ok(Some(r)) => {
                println!("CONN : we read {} bytes!", r);
                self.interest.remove(Interest::readable());
                if r > 0 {
                    event_loop.reregister(&self.non_block_client, token, self.interest,
                                          PollOpt::edge() | PollOpt::oneshot()).unwrap();
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
        match self.action {
            HttpAction::Get(ref resource) => {
                let get_command: String = String::from_str("GET ") + resource + "\n";
                let mut buf = ByteBuf::from_slice(get_command.as_bytes());
                println!("GET {}", resource);
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
                                      PollOpt::edge() | PollOpt::oneshot()).unwrap();
            }
        }
        //_ => {}
    }
    fn notify(&mut self, event_loop: &mut EventLoop<Echo>, msg: String) {
        println!("test3");
    }
}

pub fn poke_web_page(hostname: String, port: u16, action: HttpAction) {
    let mut event_loop = EventLoop::new().unwrap();
    let ip = std::net::lookup_host(&hostname).unwrap().next().unwrap().unwrap();
    let address = SocketAddr::new(ip.ip(), port);
    let (sock, _) = TcpSocket::v4().unwrap().connect(&address).unwrap();
    event_loop.register_opt(&sock, CLIENT, Interest::writable(),
                            PollOpt::edge() | PollOpt::oneshot()).unwrap();
    event_loop.run(&mut Echo::new(sock, action)).unwrap()
}
