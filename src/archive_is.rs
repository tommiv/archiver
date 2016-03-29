extern crate iron;
extern crate hyper;
extern crate regex;

use std::io::Read;
use std::time::Duration;
use std::error::Error;
use iron::prelude::*;
use hyper::Client;
use hyper::header::{Headers, UserAgent, ContentType};


const USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_11_4) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/51.0.2687.0 Safari/537.36";
const ARCHIVE_IS: &'static str = "http://archive.is/";
const TIMEOUT: u64 = 6;


header!{ (Link   , "Link"   ) => [String] }
header!{ (Refresh, "Refresh") => [String] }


// todo: use regex! macro (works only in nightly by now)
// todo: find a way to set up timeout without config parsing. const mut is not a option since it requires unsafe block


fn get_user_agent() -> UserAgent {
    return UserAgent(String::from(USER_AGENT));
}

fn set_client_defaults(client: &mut Client) {
    let timeout = Some(Duration::from_secs(TIMEOUT));

    client.set_read_timeout(timeout);
    client.set_write_timeout(timeout);
}

fn get_token() -> Result<String, String> {
    let mut client = Client::new();
    set_client_defaults(&mut client);

    let mut headers = Headers::new();
    headers.set(get_user_agent());

    let mut res = match client.get(ARCHIVE_IS).headers(headers).send() {
        Ok(resp) => resp,
        Err(why) => return Err(why.description().into())
    };

    let mut body = String::new();
    match res.read_to_string(&mut body) {
        Err(why) => return Err(why.description().into()),
        Ok(_)    => {}
    };

    let re = regex::Regex::new(r#"name="submitid" value="([^"]+)""#).unwrap();
    let cap = match re.captures(&body) {
        Some(val) => val,
        None      => return Err("Token not found".into())
    };

    return Ok(cap.at(1).unwrap().into());
}

fn submit_with_token(token: String, url: String) -> Result<String, String> {
    let mut headers = Headers::new();
    headers.set(get_user_agent());
    headers.set(ContentType::form_url_encoded());

    let form = format!("url={0}&submitid={1}", url, token);
    let action = format!("{0}submit/", ARCHIVE_IS);

    let mut client = Client::new();
    set_client_defaults(&mut client);

    let mut res = match client.post(&action).headers(headers).body(&form).send() {
        Ok(val)  => val,
        Err(why) => return Err(why.description().into())
    };

    if res.headers.has::<Refresh>() {
        let val: Vec<&str> = res.headers.get::<Refresh>().unwrap().split(";").collect();
        if val.len() == 2 {
            return Ok(String::from(val[1]).replace("url=", ""));
        }
    } else if res.headers.has::<Link>() {
        let mut body = String::new();
        match res.read_to_string(&mut body) {
            Err(why) => return Err(why.description().into()),
            Ok(_)    => {}
        };

        let re = regex::Regex::new(r#"<link\s+rel="canonical"\s+href="([^"]+)""#).unwrap();
        let caps = re.captures(&body);
        if caps.is_some() {
            return Ok(caps.unwrap().at(1).unwrap().into());
        }
    }

    return Err("Can't parse response".into());
}

pub fn handle(req: &mut Request) -> IronResult<Response> {
    let mut input_link = String::new();
    if req.body.read_to_string(&mut input_link).unwrap_or(0) == 0 {
        return Ok(Response::with((iron::status::BadRequest, "Body is empty or cannot be read")));
    }

    let token = match get_token() {
        Ok(val)  => val,
        Err(why) => return Ok(Response::with((iron::status::ServiceUnavailable, why)))
    };

    match submit_with_token(token, input_link) {
        Ok(val)  => return Ok(Response::with((iron::status::Ok, val))),
        Err(why) => return Ok(Response::with((iron::status::ServiceUnavailable, why)))
    };
}