extern crate iron;
extern crate router;
#[macro_use] extern crate hyper;

use iron::prelude::*;
use router::Router;

mod archive_is;

fn main() {
    let mut router = Router::new();
    router.get("/archive.is/:url", archive_is::handle);

    Iron::new(router).http("localhost:3000").unwrap();
}