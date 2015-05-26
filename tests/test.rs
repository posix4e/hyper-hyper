#![feature(collections)]

extern crate mio;
extern crate hyperhyper;

use std::rc::Rc;
use hyperhyper::action::poke_web_page;
use hyperhyper::action::HttpAction;

#[test]
fn get_a_simple_webpage() {
    let result:Vec<u8> = poke_web_page("google.com".to_string(), 
    	80, 
    	HttpAction::Get(Rc::new(String::from_str("/"))));
	assert!(String::from_utf8(result).unwrap().contains("google"));
}
