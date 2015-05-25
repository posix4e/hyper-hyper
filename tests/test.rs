#![feature(collections)]


extern crate mio;
use std::net::SocketAddr;

use std::str::FromStr;
use std::rc::Rc;
extern crate hyperhyper;
use hyperhyper::*;


pub fn google() -> SocketAddr {
    let s = format!("216.58.192.4: {}", 80);
    FromStr::from_str(&s).unwrap()
}

#[test]
fn test() {
    let path = String::from_str("/");
    let resource = Rc::new(path);
    get_web_page("news.ycombinator.com".to_string(), 80, HttpAction::Get(resource));
    //println!(result);
}
