use crate::request;
use tokio::sync::mpsc::UnboundedReceiver;
use serde::{Serialize,Deserialize};
use std::net::TcpStream;
use std::{
    sync::{Arc,atomic::{Ordering, AtomicUsize},Mutex},
    thread::sleep,
    time::Duration,
    io::{self,Write,Cursor, Read, BufWriter,BufRead, BufReader},
    cmp::min, fmt::format, ops::{Deref, Index}
};

type BoxError = Box<dyn std::error::Error>;

pub fn connect(addr:&str,request:request::Request<request::Login>) -> Result<TcpStream,BoxError> {
    let mut stream = TcpStream::connect(addr)?;
    stream.set_nodelay(true)?;
    serde_json::to_writer(&mut stream, &request)?;
    writeln!(&mut stream)?;
    stream.flush()?;
    Ok(stream)
}

pub struct Client {
    pub stream:TcpStream,
    pub reciever:UnboundedReceiver<request::MessageType>,
}
impl Client {
    pub fn send<'a,T>(&mut self,request:request::Request<'a,T>) -> Result<(),BoxError>
    where T:Serialize + Deserialize<'a> + std::fmt::Debug
    {
        serde_json::to_writer(&mut self.stream, &request).unwrap();
        writeln!(&mut self.stream).unwrap();
        self.stream.flush().unwrap();
        Ok(())
    }
    pub fn message_listener(&mut self) {
        loop {
            if let Some(msg) = self.reciever.blocking_recv() {
                match msg {
                    request::MessageType::Submit(submit) => {
                        println!("recieved share");
                        let request = request::Request {
                            id:1,
                            method:"submit".to_string(),
                            params:&submit
                        };
                        self.send(request).unwrap();
                        println!("submitted share");
                    },
                    request::MessageType::KeepAlive(keep_alive) => {
                        let req =  request::Request {
                            id:1,
                            method:"keepalived".to_string(),
                            params: &keep_alive
                        };
                        self.send(req).unwrap();
                    },
                }
            }
        }
    }
}
