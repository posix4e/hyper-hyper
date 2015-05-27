#![feature(collections)]

extern crate mio;
extern crate hyperhyper;
extern crate eventual;

use mio::*;
use hyperhyper::action::Echo;
use hyperhyper::action::HttpAction;
use std::thread;
use eventual::Async;
use std::str;

#[test]
fn get_a_simple_webpage_2() {
    let mut event_loop = EventLoop::new().unwrap();
    let (tx, rx) = eventual::Future::<Box<Vec<u8>>, &'static str>::pair();

    //  poke_web_page_async(event_loop,
    // 	tx,
    //  	"google.com".to_string(), 
    //   	80, 
    //    	HttpAction::Get(Rc::new(String::from_str("/"))));
    event_loop.channel().send(("test".to_string(), tx));

    thread::spawn(move || {
        let echo = &mut Echo::new();
        event_loop.run(echo);
    });

    let vec = &*rx.await().unwrap();
    assert!( str::from_utf8(vec).unwrap().contains("google"));
}
fn poke_web_page_async(mut event_loop: EventLoop<Echo>,
                       sender: eventual::Complete<Vec<u8>, &'static str>,
                       hostname: String,
                       port: u16,
                       action: HttpAction) {
    println!("poke");
}
