use std::sync::Arc;
use mio::*;
use mio::tcp::{TcpStream, TcpSocket};
use mio::buf::ByteBuf;
use std::net::SocketAddr;
use std::net::lookup_host;
use std::collections::VecMap;
use url::Url;

use eventual;

#[derive(Debug, Clone)]
pub enum HttpAction {
    Get(Arc<Url>),
}

pub struct Echo {
    non_block_client: VecMap<TcpStream>,
    action: VecMap<HttpAction>,
    sender: VecMap<eventual::Complete<Box<Vec<u8>>, &'static str>>,
    mut_buf: VecMap<Vec<u8>>,
    buf: Option<ByteBuf>,
    interest: VecMap<Interest>
}

impl Echo {
    pub fn new() -> Echo {
        Echo {
            non_block_client: VecMap::new(),
            action: VecMap::new(),
            sender: VecMap::new(),
            mut_buf: VecMap::new(),
            interest: VecMap::new(),
            buf: None,
        }
    }
}

impl Handler for Echo {
    type Timeout = usize;
    type Message = (String, eventual::Complete<Box<Vec<u8>>, &'static str>);

    fn readable(&mut self, event_loop: &mut EventLoop<Echo>, token: Token, hint: ReadHint) {
        println!("Read");
        
        let mut buf = ByteBuf::mut_with_capacity(4096 * 16);

        let ref mut non_block_client = match self.non_block_client.get_mut(&token.as_usize()) {
            Some(client) => client,
            None => panic!("Error finding the associated non blocking client {:?}", token)
        };
        let ref mut interest = match self.interest.get_mut(&token.as_usize()) {
            Some(interest) => interest,
            None => panic!("Error finding the associated interest {:?}", token)
        };
        match non_block_client.try_read_buf(&mut buf) {

            Ok(None) => {
                panic!("We just got readable, but were unable to read from the socket?");
            }
            Ok(Some(r)) => {
                interest.remove(Interest::readable());
                if r == 0 {
                	println!("DONE");
                    event_loop.deregister(*non_block_client).unwrap();
                    let mut_buf = match self.mut_buf.remove(&token.as_usize()) {
                        Some(mut_buf) => mut_buf,
                        None =>
                            panic!("Error finding the mut_buf for {:?} {:?}", token, self.mut_buf)
                    };

                    match self.sender.remove(&token.as_usize()) {
                        Some(completer) => {
                            completer.complete(Box::new(mut_buf));
                            interest.remove(Interest::none());
                        }
                        None => panic!("Error finding the mut_buf for {:?}", token)
                    };

                } else {
        event_loop.reregister(*non_block_client, token, **interest,
                              PollOpt::edge() | PollOpt::oneshot()).unwrap();
 
                    match self.mut_buf.get_mut(&token.as_usize()) {
                        Some(mut_buf) => mut_buf.push_all(buf.flip().bytes()),
                        None => panic!("Error finding the mut_buf for {:?}", token)
                    }
                    
                    interest.insert(Interest::readable());
                }
            }
            Err(e) => {
                panic!("not implemented; client err={:?}", e);
            }
        }
    }

    fn writable(&mut self, event_loop: &mut EventLoop<Echo>, token: Token) {
        let ref mut action = match self.action.get_mut(&token.as_usize()) {
            Some(action) => action,
            None => panic!("Error finding the associated action {:?}", token)
        };
        let get_command: String = body(action.clone());

        let ref mut non_block_client = match self.non_block_client.get_mut(&token.as_usize()) {
            Some(client) => client,
            None => panic!("Error finding the associated non blocking client {:?}", token)
        };

        let ref mut interest = match self.interest.get_mut(&token.as_usize()) {
            Some(interest) => interest,
            None => panic!("Error finding the associated interest {:?}", token)
        };

        match **action {
            HttpAction::Get(_) => {
                let mut buf = ByteBuf::from_slice(get_command.as_bytes());
                match non_block_client.try_write_buf(&mut buf) {
                    Ok(None) => {
                        println!("client flushing buf; WOULDBLOCK");
                        self.buf = Some(buf);
                        interest.insert(Interest::writable());
                    }
                    Ok(Some(_)) => {
                        interest.insert(Interest::readable());
                        interest.remove(Interest::writable());
                        event_loop.reregister(*non_block_client, token, **interest,
                                              PollOpt::edge() | PollOpt::oneshot()).unwrap();
                    }
                    Err(e) => panic!("not implemented; client err={:?}", e),
                }
            }
        }
    }
    fn notify(&mut self,
              event_loop: &mut EventLoop<Echo>,
              tuple: (String, eventual::Complete<Box<Vec<u8>>, &'static str>)) {
        let token = Token(self.action.len() + 1);
        let action = get_action(tuple.0);
        self.action.insert(token.as_usize(), action.clone());
               
        match action {
            HttpAction::Get(url_p) => {
                let url: Url = (*url_p).clone();
                let ip = lookup_host(url.domain().unwrap()).unwrap().next().unwrap().unwrap();
                let port = url.port_or_default().unwrap();
                let address = SocketAddr::new(ip.ip(), port);
                let (sock, _) = TcpSocket::v4().unwrap().connect(&address).unwrap();
                self.non_block_client.insert(token.as_usize(), sock);
                self.sender.insert(token.as_usize(), tuple.1);
                self.mut_buf.insert(token.as_usize(), Vec::new());
                self.interest.insert(token.as_usize(), Interest::hup());
                event_loop.register_opt(self.non_block_client.get(&token.as_usize()).unwrap(),
                                        token, Interest::writable(),
                                        PollOpt::edge() | PollOpt::oneshot()).unwrap();
            }
        }
    }
}

fn body(action: HttpAction) -> String {
    match action {
        HttpAction::Get(ref resource) => {
            format!(
                "GET {}  HTTP/1.1\r\nHost: {}\r\nUser-Agent: curl/7.37.1\r\nAccept */*\r\n\r\n",
                resource.serialize_path().unwrap(), resource.domain().unwrap())
        }
    }
}
fn get_action(url_s: String) -> HttpAction {
    let url = Url::parse(url_s.as_str()).unwrap();
    HttpAction::Get(Arc::new(url))
}