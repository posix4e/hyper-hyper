
extern crate mio;
extern crate hyperhyper;
extern crate eventual;

use mio::EventLoop;
use hyperhyper::action::Echo;
use std::thread;
use eventual::Async;
use std::str;

#[test]
fn get_a_simple_webpage_2() {
    let mut event_loop = EventLoop::new().unwrap();
    let (tx_google, rx_google) = eventual::Future::<Box<Vec<u8>>, &'static str>::pair();
    let (tx_reddit, rx_reddit) = eventual::Future::<Box<Vec<u8>>, &'static str>::pair();

    //  poke_web_page_async(event_loop,
    // 	tx,
    //  	"google.com".to_string(), 
    //   	80, 
    //    	HttpAction::Get(Rc::new(String::from_str("/"))));
    event_loop.channel().send(("http://rust.wuli.nu".to_string(), tx_google)).unwrap();
	event_loop.channel().send(("http://www.google.com/".to_string(), tx_reddit)).unwrap();

    thread::spawn(move || {
        let echo = &mut Echo::new();
        event_loop.run(echo).unwrap();
    });

	println!("Getting google");
	let google = &*rx_google.await().unwrap();
	println!("Got google");
	let reddit = &*rx_reddit.await().unwrap();

	assert!(str::from_utf8(google).unwrap().contains("google"));
	assert!(str::from_utf8(reddit).unwrap().contains("reddit"));
	
}