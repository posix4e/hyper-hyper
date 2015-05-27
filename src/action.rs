use std::sync::Arc;
use mio::*;
use mio::tcp::{TcpStream, TcpSocket};
use mio::buf::ByteBuf;
use std::net::SocketAddr;
use std::net::lookup_host;
use std::collections::VecMap;
use eventual;

#[derive(Debug)]
pub enum HttpAction {
    Get(Arc<String>),
}

pub struct Echo {
    non_block_client: VecMap<TcpStream>,
    action: VecMap<HttpAction>,
    sender: VecMap<eventual::Complete<Box<Vec<u8>>, &'static str>>,
    mut_buf: VecMap<Vec<u8>>,
    buf: Option<ByteBuf>,
    interest: Interest
}

impl Echo {
    pub fn new() -> Echo {
        Echo {
            non_block_client: VecMap::new(),
            action: VecMap::new(),
            sender: VecMap::new(),
            mut_buf: VecMap::new(),
            interest: Interest::hup(),
            buf: None,
        }
    }
}

impl Handler for Echo {
    type Timeout = usize;
    type Message = (String, eventual::Complete<Box<Vec<u8>>, &'static str>);

    fn readable(&mut self, event_loop: &mut EventLoop<Echo>, token: Token, _: ReadHint) {
        let mut buf = ByteBuf::mut_with_capacity(2048);

        let ref mut non_block_client = match self.non_block_client.get_mut(&token.as_usize()) {
            Some(client) => client,
            None => panic!("Error finding the associated non blocking client {:?}", token)
        };

        match non_block_client.read(&mut buf) {
            Ok(None) => {
                println!("We just got readable, but were unable to read from the socket?");
            }
            Ok(Some(r)) => {
                self.interest.remove(Interest::readable());
                if r == 0 {

                    let mut_buf = match self.mut_buf.remove(&token.as_usize()) {
                        Some(mut_buf) => mut_buf,
                        None =>
                            panic!("Error finding the mut_buf for {:?}", token)
                    };
                    
                    match self.sender.remove(&token.as_usize()) {
                        Some(completer) => completer.complete(Box::new(mut_buf)),
                        None =>
                            panic!("Error finding the mut_buf for {:?}", token)
                    };
					
						
                } else {
                    let ref mut mut_buf = match self.mut_buf.get_mut(&token.as_usize()) {
                        Some(mut_buf) => mut_buf,
                        None =>
                            panic!("Error finding the mut_buf for {:?}", token)
                    };
					mut_buf.push_all(buf.flip().bytes());
                    event_loop.reregister(*non_block_client, token, self.interest,
                                          PollOpt::edge() | PollOpt::oneshot()).unwrap();
                }
            }
            Err(e) => {
                println!("not implemented; client err={:?}", e);
                self.interest.remove(Interest::readable());
            }
        }
    }

    fn writable(&mut self, event_loop: &mut EventLoop<Echo>, token: Token) {
        let ref mut action = match self.action.get_mut(&token.as_usize()) {
            Some(action) => action,
            None => panic!("Error finding the associated action {:?}", token)
        };

        let ref mut non_block_client = match self.non_block_client.get_mut(&token.as_usize()) {
            Some(client) => client,
            None => panic!("Error finding the associated non blocking client {:?}", token)
        };

        match **action {
            HttpAction::Get(ref resource) => {
                let get_command: String = String::from_str("GET ") + resource + "\n";
                let mut buf = ByteBuf::from_slice(get_command.as_bytes());
                println!("GET {}", resource);
                match non_block_client.write(&mut buf) {
                    Ok(None) => {
                        println!("client flushing buf; WOULDBLOCK");
                        self.buf = Some(buf);
                        self.interest.insert(Interest::writable());
                    }
                    Ok(Some(_)) => {
                        self.interest.insert(Interest::readable());
                        self.interest.remove(Interest::writable());
                    }
                    Err(e) => panic!("not implemented; client err={:?}", e),
                }
                event_loop.reregister(*non_block_client, token, self.interest,
                                      PollOpt::edge() | PollOpt::oneshot()).unwrap();
            }
        }
    }
    fn notify(&mut self,
              event_loop: &mut EventLoop<Echo>,
              tuple: (String, eventual::Complete<Box<Vec<u8>>, &'static str>)) {

        let token = Token(self.action.len() + 1);
        println!("notify token {:?}", token);

        let url_tuple = url_tuple(tuple.0);
        let ip = lookup_host(&url_tuple.0).unwrap().next().unwrap().unwrap();
        let address = SocketAddr::new(ip.ip(), url_tuple.1);
        let (sock, _) = TcpSocket::v4().unwrap().connect(&address).unwrap();
        self.non_block_client.insert(token.as_usize(), sock);
        self.action.insert(token.as_usize(), url_tuple.2);
        self.sender.insert(token.as_usize(), tuple.1);
        self.mut_buf.insert(token.as_usize(), Vec::new());
        event_loop.register_opt(self.non_block_client.get(&token.as_usize()).unwrap(), token,
                                Interest::writable(), PollOpt::edge() | PollOpt::oneshot())
                .unwrap();
    }
}

fn url_tuple(_: String) -> (String, u16, HttpAction) {
    ("google.com".to_string(), 80, HttpAction::Get(Arc::new("/".to_string())))
}