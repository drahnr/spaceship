extern crate time;

use time::Timespec;

use std::error::Error;
use std::io::prelude::*;
use std::fs::File;
use std::path::Path;

#[derive(Debug)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub body: String,
    pub date: Timespec,
    pub slug: String,
}

impl Default for Post {
	fn default() -> Post {
		Post {
			id: 0,
			title: String::from(""),
			body: String::from(""),
			date: Timespec::new(0,0),
			slug: String::from("")
		}
	}
}
