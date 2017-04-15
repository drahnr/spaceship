use Post;

extern crate postgres;

use postgres::{Connection, TlsMode};
use std::collections::HashMap;

extern crate time;

use time::Timespec;

#[derive(Debug)]
struct BackendError(&'static str);

pub trait Backend {
	fn store(self, post : &Post) -> Result<(),BackendError>;
	fn load(self, slug : String) -> Result<Post,BackendError>;
}

pub struct PostgresBackend {
	conn : Connection,
}

impl PostgresBackend {
	fn new() -> PostgresBackend {
		PostgresBackend{ conn: None}
	}

	fn connect(self) -> Result<(),BackendError> {
		let user = String::from("username");
		let hostname = String::from("127.0.0.1");
		let conn = match Connection::connect(format!("postgres://{user}@{host}", user=user, host=hostname), TlsMode::None) {
			Ok(c) => c,
			Err(e) => BackendError("Failed to setup postgres connection: {:?}", e)
		};
		self.conn = conn;
		self.conn.execute("CREATE TABLE IF NOT EXISTS posts (
		                id              SERIAL PRIMARY KEY,
		                title            VARCHAR NOT NULL,
		                body            VARCHAR NOT NULL,
		                date			TIMESTAMP,
		                slug			VARCHAR NOT NULL UNIQUE
		              )", &[])
	}
	fn disconnect(self) -> Result<(),BackendError>  {
		Ok(())
	}

}

impl Backend for PostgresBackend {
	fn store(self, post : &Post) -> Result<(),BackendError> {
		self.conn.execute("INSERT INTO posts (title, body, date, slug) VALUES ($1, $2, $3, $4)",
		             &[&post.title, &post.body, &post.date, &post.slug])?;
		Ok(())
	}

	fn load(self, slug : String) -> Result<Post,BackendError> {
		let rows = self.conn.query("SELECT id, title, body, date, slug FROM post WHERE post.slug == $1", &[slug.as_str()])?;
		let row = &rows.first().ok_or(BackendError("No such db record"))?;
		Ok(Post{
		        id: row.get(0),
		        title: row.get(1),
		        body: row.get(2),
		        date: row.get(3),
		        slug: row.get(4),
		})
	}
}


struct Cache<B> where B: Backend {
	backend : B,
	slug_to_content : HashMap<String, String>,
}

impl Cache<B> where B: Backend {
	fn new(backend: B) -> Cache<B> {
		Cache{backend: backend, slug_to_content: HashMap::new(),}
	}
}

impl Backend for Cache<B> where B: Backend {
	fn store(self, post : &Post) -> Result<(),BackendError> {
		let result = self.backend.store(post);
		let _ = self.slug_to_content.remove(&post.slug).map_err(Ok(()));
		result
	}

	fn load(self, slug : String) -> Result<Post,BackendError> {
		match self.slug_to_content.get(&slug) {
			Some(content) => Ok(Post::default()), // FIXME
			None => {
				self.backend.load(slug).and_then(|x| self.slug_to_content.insert(slug, x))
			}
		}
	}
}
