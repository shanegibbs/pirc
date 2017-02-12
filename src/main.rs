#![feature(conservative_impl_trait)]
extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

mod codec;

use codec::IrcCodec;
use futures::{Future, Sink, Stream, stream};
use futures::future::IntoFuture;
use std::net::ToSocketAddrs;
use tokio_core::io::Io;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "localhost:6667".to_socket_addrs().unwrap().next().unwrap();

    let work = TcpStream::connect(&addr, &handle)
        .and_then(|socket| {
            let transport = socket.framed(IrcCodec::new());
            let lines_to_send: Vec<Result<String, std::io::Error>> =
                vec![Ok("NICK bot".to_string()), Ok("USER bot 8 * :Bot".to_string())];
            transport.send_all(stream::iter(lines_to_send))
        })
        .and_then(|(transport, _results)| {
            transport.for_each(|l| {
                println!("{:?}", l);
                if l.contains("PING") {
                    // TODO how do i send back a PONG?
                }
                Ok(())
            })
        })
        .and_then(|_| Ok(()));

    core.run(work).unwrap();
}
