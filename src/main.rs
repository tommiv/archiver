extern crate iron;
extern crate router;
extern crate logger;
extern crate config;
extern crate unicase;
#[macro_use] extern crate hyper;

use std::path::{ Path, PathBuf };
use iron::prelude::*;
use unicase::UniCase;

mod archive_is;


fn main() {
    let mut args = std::env::args();
    let cfg_path = args.nth(1).unwrap_or("config.cfg".into());
    let conf = config::reader::from_file(Path::new(&cfg_path)).unwrap();

    let mut router = router::Router::new();
    router.post("/archive.is", archive_is::handle);

    let mut chain = Chain::new(router);

    let (logger_before, logger_after) = logger::Logger::new(None);
    chain.link_before(logger_before);
    chain.link_after(logger_after);

    chain.link_after(CorsMiddleware);

    let listen = conf.lookup_str("listen").unwrap();
    let threads = conf.lookup_integer32("threads").unwrap() as usize;
    let protocol = iron::Protocol::Https {
        certificate: PathBuf::from(conf.lookup_str("ssl_cer").unwrap()),
        key        : PathBuf::from(conf.lookup_str("ssl_key").unwrap())
    };

    let iron = Iron::new(chain).listen_with(listen, threads, protocol, None);

    if iron.is_err() {
        println!("{}", "Well, fug.");
    }
}


struct CorsMiddleware;

impl iron::AfterMiddleware for CorsMiddleware {
    fn after(&self, _: &mut Request, mut res: Response) -> IronResult<Response> {
        res.headers.set(hyper::header::AccessControlAllowOrigin::Any);
        res.headers.set(
            hyper::header::AccessControlAllowHeaders(vec![
                UniCase("Content-Type".to_owned())
            ])
        );
        Ok(res)
    }
}