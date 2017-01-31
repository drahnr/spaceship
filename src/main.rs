#[macro_use] extern crate tera;


use tera::Tera;
use tera::Context;


extern crate iron;
extern crate router;

use iron::prelude::*;
use router::Router;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
	ctx.add("title", &"<b>spacship!</b>");
	ctx.add("body", &"it's working");

	let obj = match spaceship.tera().lock() {
        Ok(x) => x,
        Err(_) => return Ok(Response::with((iron::status::InternalServerError)))
    };
	let content = obj.render("templates/index.html", ctx).unwrap();
	return Ok(Response::with((iron::status::Ok, content)));
}

fn main() {
    let spaceship = Spaceship{tera: Arc::new(Mutex::new(compile_templates!("templates/**/*"))), ..Default::default()};
    let mut router = Router::new();

    router.get("/", move |request: &mut Request| index(request, &spaceship), "magicid");

    Iron::new(router).http("localhost:8080").unwrap();
}
