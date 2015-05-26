#![crate_type = "lib"]
#![crate_name = "hyperhyper"]
#![feature(collections)]
#![feature(lookup_host)]
#![feature(ip_addr)]

extern crate mio;
use mio::*;
use mio::tcp::*;
use std::net::SocketAddr;

use mio::buf::ByteBuf;
use std::rc::Rc;
use mio::util::Slab;

// Define a handler to process the events
const CLIENT: Token = Token(1);

#[derive(Debug)]
pub enum HttpAction {
    Get(Rc<String>),
}

pub struct Echo {
    non_block_client: TcpStream,
    action: HttpAction,
    slab: Vec<u8>,
    buf: Option<ByteBuf>,
    interest: Interest
}

impl Echo {
    pub fn new(client: TcpStream, action: HttpAction) -> Echo {
        Echo {
            non_block_client: client,
            action: action,
            slab: Vec::new(),
            interest: Interest::hup(),
            buf: None,
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
                self.interest.remove(Interest::readable());
                if r > 0 {
                	self.slab.push_all(buf.flip().bytes());
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
                        self.interest.insert(Interest::readable());
                        self.interest.remove(Interest::writable());
                    }
                    Err(e) => panic!("not implemented; client err={:?}", e),
                }
                event_loop.reregister(&self.non_block_client, token, self.interest,
                                      PollOpt::edge() | PollOpt::oneshot()).unwrap();
            }
        }
        //_ => {}
    }
    fn notify(&mut self, event_loop: &mut EventLoop<Echo>, msg: String) {

    }
}

pub fn poke_web_page(hostname: String, port: u16, action: HttpAction) {
    let mut event_loop = EventLoop::new().unwrap();
    let ip = std::net::lookup_host(&hostname).unwrap().next().unwrap().unwrap();
    let address = SocketAddr::new(ip.ip(), port);
    let (sock, _) = TcpSocket::v4().unwrap().connect(&address).unwrap();
    event_loop.register_opt(&sock, CLIENT, Interest::writable(),
                            PollOpt::edge() | PollOpt::oneshot()).unwrap();
    let echo = &mut Echo::new(sock, action);
    event_loop.run(echo).unwrap();
    
//    let s = match str::from_utf8(buf) {
 //       Ok(v) => v,
//        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
   // };

	let result = String::from_utf8(echo.slab.clone());
	println!("{}", result.unwrap());
    
}
