#[macro_use] extern crate tera;

#[macro_use] extern crate lazy_static;

extern crate iron;
extern crate router;
extern crate hyper;
extern crate regex;


use std::env;
use std::io::prelude::*;
use std::fs::File;


use tera::Tera;
use tera::Context;




use regex::Regex;

use iron::prelude::*;
use router::Router;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use hyper::header::ContentType;
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
	fn cache<'a>(&'a self) -> &'a Arc<Mutex<HashMap<String, String>>> {
		&self.cache
	}
}

fn css(req: &Request, spaceship: &Spaceship) -> IronResult<Response> {
	let mut cache = match spaceship.cache().lock() {
		Ok(x) => x,
        Err(_) => return Ok(Response::with((iron::status::InternalServerError)))
    };

	let url = &req.url;
	println!("Css Request {:?}", url);
	println!("Css Request {:?}", url.path());

	let key = String::from("style.css");
	let css_str = match cache.get(&key) {
        Some(x) => x.clone(),
        None => {
			let mut f = File::open("static/style.css").unwrap();
			let mut s = String::new();
			match f.read_to_string(&mut s) {
				Err(_) => {
					println!("Could not find static/style.css");
					return Ok(Response::with((iron::status::NotFound)));
				},
				Ok(_) => {}
			};
			s
        }
    };

	cache.insert(key, css_str.clone());

	let mut resp = Response::with((iron::status::Ok, css_str));
	resp.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Css, vec![])));
	Ok(resp)
}

fn index(req: &mut Request, spaceship: &Spaceship) -> IronResult<Response> {
	lazy_static! {
		static ref CSS_REGEX: Regex = Regex::new(r".*css").unwrap();
	}

	let filename;
	let url = &req.url;
	println!("Request {:?}", url);
	println!("Request {:?}", url.path());
	filename = match url.path().last() {
		Some(x) => x.clone(),
		None => {
			println!("No filename given");
			return Ok(Response::with((iron::status::NotFound)))
		}
	};
	println!("{}", filename);
	if CSS_REGEX.is_match(filename) {
		println!("Regex match! going for CSS");
		let x = css(&req, &spaceship);
		return x;
	}


	let mut ctx = Context::new();
	ctx.add("title", &"spaceship!");
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

	let address = env::var("SPACESHIP_ADDRESS").unwrap_or(String::from("0.0.0.0"));
	let port = env::var("SPACESHIP_PORT").unwrap_or(String::from("8080"));

    let spaceship = Spaceship{tera: Arc::new(Mutex::new(compile_templates!("templates/**/*"))), ..Default::default()};
    let mut router = Router::new();
    {
    let spaceship = spaceship.clone();
    router.get("/*", move |request: &mut Request| index(request, &spaceship), "magicid");
    }
    {
    let spaceship = spaceship.clone();
    router.get("/", move |request: &mut Request| index(request, &spaceship), "index");
    }
    {
    let spaceship = spaceship.clone();
    router.get("/static/*", move |request: &mut Request| css(request, &spaceship), "static");
    }
    Iron::new(router).http(format!("{}:{}",address,port)).unwrap();
}
