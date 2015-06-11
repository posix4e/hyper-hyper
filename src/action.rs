extern crate curl;
use std::sync::Arc;
use mio::*;
use mio::tcp::{TcpStream, TcpSocket};
use mio::buf::ByteBuf;
use std::net::SocketAddr;
use std::net::lookup_host;
use std::collections::VecMap;
use url::Url;
use curl::http;
use eventual;

#[derive(Debug, Clone)]
pub enum HttpAction {
    Get(Arc<Url>),
}

pub struct ClientInfo {
    tcp_stream: TcpStream,
    action: HttpAction,
    complete: eventual::Complete<Box<Vec<u8>>, &'static str>,
    mut_buf: Vec<u8>
}

impl ClientInfo {

    pub fn complete(self) {
        self.complete.complete(Box::new(self.mut_buf));
    }
}

pub struct Echo {
    client_info: VecMap<ClientInfo>,
}

impl Echo {
    pub fn new() -> Echo {
        Echo {
            client_info: VecMap::new(),
        }
    }
}

impl Handler for Echo {
    type Timeout = usize;
    type Message = (String, eventual::Complete<Box<Vec<u8>>, &'static str>);

    fn readable(&mut self, event_loop: &mut EventLoop<Echo>, token: Token, hint: ReadHint) {
        let mut closed = false;

        if !hint.is_hup() {
            let mut buf = ByteBuf::mut_with_capacity(4096 * 16);
            let mut client_info = self.client_info.get_mut(&token.as_usize()).unwrap();
            match client_info.tcp_stream.try_read_buf(&mut buf).unwrap() {
                Some(r) => {
                    if r > 0 {
                        client_info.mut_buf.push_all(buf.flip().bytes());

                        event_loop.reregister(&client_info.tcp_stream, token, Interest::readable(),
                                              PollOpt::edge() | PollOpt::oneshot()).unwrap();
                    } else {
                        event_loop.deregister(&client_info.tcp_stream).unwrap();
                        closed = true;
                    }
                }
                _ => {
                    panic!("err!");
                }
            }

        } else {
            let client_info = self.client_info.remove(&token.as_usize()).unwrap();

            event_loop.deregister(&client_info.tcp_stream).unwrap();
            closed = true;
        }
        if closed {
            self.client_info.remove(&token.as_usize()).unwrap().complete();
        }
    }

    fn writable(&mut self, event_loop: &mut EventLoop<Echo>, token: Token) {
        let client_info = self.client_info.get_mut(&token.as_usize()).unwrap();
        let get_command: String = body(client_info.action.clone());
        match client_info.action {
            HttpAction::Get(_) => {
                let mut buf = ByteBuf::from_slice(get_command.as_bytes());
                match client_info.tcp_stream.try_write_buf(&mut buf) {
                    Ok(None) => {
                        panic!("client flushing buf; WOULDBLOCK");
                        //   self.buf = Some(buf);
                    }
                    Ok(Some(_)) => {}
                    Err(e) => panic!("not implemented; client err={:?}", e),
                }

                event_loop.reregister(&client_info.tcp_stream, token, Interest::readable(),
                                      PollOpt::edge() | PollOpt::oneshot()).unwrap();
            }
        }
    }
    fn notify(&mut self,
              event_loop: &mut EventLoop<Echo>,
              tuple: (String, eventual::Complete<Box<Vec<u8>>, &'static str>)) {
        let token = Token(self.client_info.len() + 1);

        let action = get_action(tuple.0);
        match action.clone() {
            HttpAction::Get(url_p) => {
                let url: Url = (*url_p).clone();
                let ip = lookup_host(url.domain().unwrap()).unwrap().next().unwrap().unwrap();
                let port = url.port_or_default().unwrap();
                let address = SocketAddr::new(ip.ip(), port);
                let (sock, _) = TcpSocket::v4().unwrap().connect(&address).unwrap();
                let client_info = ClientInfo {
                    tcp_stream: sock,
                    action: action,
                    complete: tuple.1,
                    mut_buf: Vec::new()
                };

                event_loop.register_opt(&client_info.tcp_stream, token, Interest::writable(),
                                        PollOpt::edge() | PollOpt::oneshot()).unwrap();
                self.client_info.insert(token.as_usize(), client_info);
            }
        }
    }
}

fn body(action: HttpAction) -> String {
    match action {
        HttpAction::Get(ref resource) => {
            format!(
                "GET {}  HTTP/1.1\r\nHost: {}\r\nUser-Agent: curl/7.37.1\r\nAccept */*\r\nConnection: close \r\n\r\n",
                resource.serialize_path().unwrap(), resource.domain().unwrap())
        }
    }
}
fn get_action(url_s: String) -> HttpAction {
    let htt = http::multi_handle();
    let url = Url::parse(url_s.as_str()).unwrap();
    HttpAction::Get(Arc::new(url))
}
