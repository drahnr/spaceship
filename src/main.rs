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


use std::error::Error;
use std::fmt::{self, Debug};

mod post;
use post::*;


mod backend;
use backend::*;

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str { &*self.0 }
}



#[derive(Default,Clone)]
struct Spaceship {
    cache: Arc<RwLock<Cache>>,
    tera: Arc<RwLock<Tera>>,
}

impl Spaceship {
	fn new() -> Spaceship {
		Spaceship{tera: Arc::new(RwLock::new(compile_templates!("templates/**/*"))), ..Default::default()}
	}
	fn tera<'a>(&'a self) -> &'a Arc<RwLock<Tera>> {
		&self.tera
	}
	fn cache<'a>(&'a self) -> &'a Arc<RwLock<Cache>> {
		&self.cache
	}

	fn render(&self, template: &String, ctx: &mut Context) -> Option<String> {
		match self.tera().read() {
			Ok(x) => {
				x.render(template, ctx).and_then(|content| {Ok(content)}).ok()
				},
			Err(_) => None,
		}
	}
}

impl Handler for Spaceship {
	fn handle(&self, req: &mut Request) -> IronResult<Response> {
		let mut chunks = req.url.path();
		chunks.retain(|&x| x != "");
		let path = chunks.join("/");

		let host = req.url.host();

		let hostname = match host {
            iron::url::Host::Domain(ref host) => String::from(*host),
            iron::url::Host::Ipv4(addr) => format!("{}", addr),
            iron::url::Host::Ipv6(addr) => format!("{}", addr),
		};

		let mut ctx = Context::new();
		ctx.add("title", &"spaceship!");
		ctx.add("host", &hostname);


		let template = String::from("workinprogress.html");
		match self.render(&template, &mut ctx) {
			Some(content) => {
				let mut resp = Response::with((iron::status::Ok, content));
				resp.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
				return Ok(resp);
			},
			None => {},
		}
		Err(IronError::new(StringError("Error".to_string()), iron::status::InternalServerError))
	}
}


impl AfterMiddleware for Spaceship {
    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
		let mut chunks = req.url.path();
		chunks.retain(|&x| x != "");
		let path = chunks.join("/");

		let mut ctx = Context::new();
		ctx.add("title", &"spaceship!");
		ctx.add("body", &"404");

		let template = String::from("error.html");
		match self.render(&template, &mut ctx) {
			Some(content) => {
				let mut resp = Response::with((iron::status::Ok, content));
				resp.headers.set(ContentType(Mime(TopLevel::Text, SubLevel::Html, vec![])));
				return Ok(resp);
			},
			None => {
				return Ok(Response::with((iron::status::InternalServerError)));
			}
		}

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
			None => {}		}
		Err(IronError::new(StringError("Error".to_string()), iron::status::BadRequest))
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

    let mut router = Router::new();

	let css_handler = CssHandler::new();
    router.get("/static/*", css_handler, "css");

    let spaceship_handler = Spaceship::new();
    // router.get("/[^static]*", spaceship_handler.clone(), "spaceship");
    router.get("/", spaceship_handler.clone(), "index");

    let mut chain = Chain::new(router);
    // chain.link_before(MyMiddleware);
    chain.link_after(spaceship_handler.clone());

    Iron::new(chain).http(format!("{}:{}",address,port)).unwrap();
}
