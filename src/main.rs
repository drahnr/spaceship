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

use iron::{BeforeMiddleware, AroundMiddleware, AfterMiddleware, Handler};

use router::{Router, NoRoute};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use hyper::header::ContentType;
use hyper::mime::{Mime, TopLevel, SubLevel};


struct Custom404;

impl AfterMiddleware for Custom404 {
    fn catch(&self, _: &mut Request, err: IronError) -> IronResult<Response> {
        if let Some(_) = err.error.downcast::<NoRoute>() {
            Ok(Response::with((iron::status::NotFound, "Custom 404 response")))
        } else {
            Err(err)
        }
    }
}


#[derive(Default,Clone)]
struct Spaceship {
    cache: Arc<RwLock<HashMap<String, String>>>,
    tera: Arc<RwLock<Tera>>,
}

impl Spaceship {
	fn new() -> Spaceship {
		Spaceship{tera: Arc::new(RwLock::new(compile_templates!("templates/**/*"))), ..Default::default()}
	}
	fn tera<'a>(&'a self) -> &'a Arc<RwLock<Tera>> {
		&self.tera
	}
	fn cache<'a>(&'a self) -> &'a Arc<RwLock<HashMap<String, String>>> {
		&self.cache
	}
}

impl Handler for Spaceship {

	fn handle(&self, req: &mut Request) -> IronResult<Response> {
		let mut ctx = Context::new();
		ctx.add("title", &"spaceship!");
		ctx.add("body", &"it's working");

		let obj = match self.tera().read() {
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
}



#[derive(Default,Clone)]
struct CssHandler {
    cache: Arc<RwLock<HashMap<String, String>>>
}


impl CssHandler {
	fn new() -> CssHandler {
		CssHandler { ..Default::default()}
	}

	fn read_css_to_string(&self, path: &String) -> Option<String> {
		lazy_static! {
			static ref CSS_REGEX: Regex = Regex::new(r".*\.css").unwrap();
		}
		if CSS_REGEX.is_match(path) {
			return File::open(path).and_then(|mut f| {
						let mut s = String::new();
						f.read_to_string(&mut s).and_then(|_| {Ok(s)})
					}).ok();
		}
		None
	}
}


impl Handler for CssHandler {
	fn handle(&self, req: &mut Request) -> IronResult<Response> {
		let mut chunks = req.url.path();
		chunks.retain(|&x| x != "");
		let path = chunks.join("/");
		println!("lookup {:?} {}", chunks, path);
		match self.read_css_to_string(&String::from(path)) {
			Some(content) => {
				let mut resp = Response::with((iron::status::Ok, content));
				resp.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Css, vec![])));
				return Ok(resp);
			},
			None => {}
		}
		Ok(Response::with((iron::status::InternalServerError)))
	}
}

impl AroundMiddleware for CssHandler {
    fn around(self, handler: Box<Handler>) -> Box<Handler> {
		Box::new(CssHandler::new()) as Box<Handler>
    }
}


fn main() {
	let address = env::var("SPACESHIP_ADDRESS").unwrap_or(String::from("0.0.0.0"));
	let port = env::var("SPACESHIP_PORT").unwrap_or(String::from("8080"));

    let spaceship = Spaceship::new();
    let mut router = Router::new();

	let css_handler = CssHandler::new();
    router.get("/static/*", css_handler, "css");

    Iron::new(router).http(format!("{}:{}",address,port)).unwrap();
}
