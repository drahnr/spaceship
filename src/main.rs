#[macro_use] extern crate tera;


use std::env;


use tera::Tera;
use tera::Context;


extern crate iron;
extern crate router;
extern crate hyper;

use iron::prelude::*;
use router::Router;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use hyper::header::{Headers, ContentType};
use hyper::mime::{Mime, TopLevel, SubLevel};


#[derive(Default,Clone)]
struct Spaceship {
    cache: Arc<Mutex<HashMap<String, String>>>,
    tera: Arc<Mutex<Tera>>,
}

impl Spaceship {
	fn tera<'a>(&'a self) -> &'a Arc<Mutex<Tera>> {
		&self.tera
	}
}

fn index(_: &mut Request, spaceship: &Spaceship) -> IronResult<Response> {
	let mut ctx = Context::new();
	ctx.add("title", &"<b>spaceship!</b>");
	ctx.add("body", &"it's working");

	let obj = match spaceship.tera().lock() {
        Ok(x) => x,
        Err(_) => return Ok(Response::with((iron::status::InternalServerError)))
    };
	let content = match obj.render("index.html", ctx) {
		Ok(x) => x,
		Err(_) => return Ok(Response::with((iron::status::InternalServerError)))
	};
	let mut resp = Response::with((iron::status::Ok, content));
	resp.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
	Ok(resp)
}

fn main() {

	let address = match env::var("SPACESHIP_ADDRESS") {
		Ok(x) => x,
		Err(_) => String::from("127.0.0.1"),
	};
	let port = match env::var("SPACESHIP_PORT") {
		Ok(x) => x,
		Err(_) => String::from("8080"),
	};

    let spaceship = Spaceship{tera: Arc::new(Mutex::new(compile_templates!("templates/**/*"))), ..Default::default()};
    let mut router = Router::new();

    router.get("/", move |request: &mut Request| index(request, &spaceship), "magicid");

    Iron::new(router).http(format!("{}:{}",address,port)).unwrap();
}
