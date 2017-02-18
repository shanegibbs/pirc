use std;
use std::io::{self, Error, ErrorKind};
use tokio_core::io::{Codec, EasyBuf};

pub struct IrcCodec {
    buf: EasyBuf,
}

impl IrcCodec {
    pub fn new() -> Self {
        IrcCodec { buf: EasyBuf::new() }
    }
}

impl Codec for IrcCodec {
    type In = String;
    type Out = String;
    fn decode(&mut self, buf: &mut EasyBuf) -> io::Result<Option<Self::In>> {
        let mut b = self.buf.get_mut();
        b.extend_from_slice(buf.as_slice());
        if let Some(i) = buf.as_slice().iter().position(|&b| b == b'\n') {
            let line = buf.drain_to(i - 1);
            buf.drain_to(2);
            match std::str::from_utf8(line.as_ref()) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(Error::new(ErrorKind::Other, "invalid UTF-8")),
            }
        } else {
            Ok(None)
        }
    }
    fn encode(&mut self, msg: Self::Out, buf: &mut Vec<u8>) -> io::Result<()> {
        buf.extend(msg.as_bytes());
        buf.push(b'\n');
        Ok(())
    }
}
