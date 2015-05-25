#![feature(collections)]

extern crate mio;
use std::rc::Rc;
extern crate hyperhyper;
use hyperhyper::poke_web_page;
use hyperhyper::HttpAction;

#[test]
fn get_a_simple_webpage() {
    poke_web_page("news.ycombinator.com".to_string(), 
    	80, 
    	HttpAction::Get(Rc::new(String::from_str("/"))));
}
