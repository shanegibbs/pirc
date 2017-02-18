#![feature(conservative_impl_trait)]
extern crate futures;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;

mod codec;

use codec::IrcCodec;
use futures::{BoxFuture, Future, Sink, Stream, stream};
use futures::stream::BoxStream;
use futures::sync::mpsc::channel;
use std::net::ToSocketAddrs;
use tokio_core::io::{Framed, Io};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Core;

fn stdin() -> BoxStream<String, std::io::Error> {
    use std::thread;
    use std::io::BufRead;
    use std::io::{Error, ErrorKind};

    let (mut tx, rx) = channel(0);
    thread::spawn(move || {
        let input = std::io::stdin();
        for line in input.lock().lines() {
            match tx.send(line).wait() {
                Ok(s) => tx = s,
                Err(_) => break, // channel closed
            }
        }
    });
    return rx.then(|r| match r {
            Ok(r) => {
                r
            }
            Err(_) => Err(Error::new(ErrorKind::Other, "oh no!")),
        })
        .boxed();
}

fn init_connection<T>
    (socket: T)
     -> BoxFuture<(Framed<T, IrcCodec>, BoxStream<String, std::io::Error>), std::io::Error>
    where T: Io + Send + 'static,
{
    let transport = socket.framed(IrcCodec::new());
    let lines_to_send: Vec<Result<String, std::io::Error>>;
    lines_to_send = vec![Ok("NICK rusty-bot".to_string()),
                         Ok("USER rusty-bot 8 * :Bot".to_string()),
                         Ok("JOIN #main".to_string())];
    transport.send_all(stream::iter(lines_to_send))
        .map(|(transport, results)| (transport, results.boxed()))
        .boxed()
}

fn auto_response<S>(strm: S) -> impl Stream<Item = String, Error = std::io::Error>
    where S: Stream<Item = String, Error = std::io::Error>,
{
    strm.filter_map(|l| {
        println!("{:?}", l);
        if l.contains("PING") {
            let response = "PONG :irc.example.net".to_string();
            println!("Sending {:?}", response);
            Some(response)
        } else {
            None
        }
    })
}

fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    let addr = "localhost:6667".to_socket_addrs().unwrap().next().unwrap();

    let work = TcpStream::connect(&addr, &handle)
        .and_then(init_connection)
        .and_then(|(transport, _results)| {
            let (sink, strm) = transport.split();
            sink.send_all(auto_response(strm).select(stdin()))
        })
        .and_then(|_| Ok(()));

    let r = core.run(work);
    println!("{:?}", r);
}
