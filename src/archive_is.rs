extern crate iron;
extern crate hyper;
extern crate regex;

use std::io::Read;
use iron::prelude::*;
use router::Router;
use hyper::Client;
use hyper::header::{Headers, UserAgent, ContentType};


const USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2687.0 Safari/537.36";
const ARCHIVE_IS: &'static str = "http://archive.is/";

header!{ (Link   , "Link"   ) => [String] }
header!{ (Refresh, "Refresh") => [String] }

// #todo link parse, use post, json, timeouts, error handling, log


fn get_user_agent() -> UserAgent {
    return UserAgent(String::from(USER_AGENT));
}

fn get_token() -> String {
    let mut headers = Headers::new();
    headers.set(get_user_agent());

    let mut res = Client::new()
        .get(ARCHIVE_IS)
        .headers(headers)
        .send()
        .unwrap();

    let mut body = String::new();
    res.read_to_string(&mut body).unwrap();

    let re = regex::Regex::new("name=\"submitid\" value=\"([^\"]+)\"").unwrap();
    let cap = re.captures(&body).unwrap();

    return String::from(cap.at(1).unwrap());
}

fn submit_with_token(token: String, url: String) -> String {
    let mut headers = Headers::new();
    headers.set(get_user_agent());
    headers.set(ContentType::form_url_encoded());

    let form = format!("url={0}&submitid={1}", url, token);
    let action = format!("{0}submit/", ARCHIVE_IS);

    let mut res = Client::new()
        .post(&action)
        .headers(headers)
        .body(&form)
        .send()
        .unwrap();

    if (res.headers.has::<Refresh>()) {

    } else if (res.headers.has::<Link>()) {
        let mut body = String::new();
        res.read_to_string(&mut body).unwrap();
    } else {

    }

    return String::new();
}

pub fn handle(req: &mut Request) -> IronResult<Response> {
    let token = get_token();

    let url = String::from(req.extensions.get::<Router>().unwrap().find("url").unwrap());

    let body = submit_with_token(token, url);

    return Ok(Response::with((iron::status::Ok, body)));
}