mod protocol;
mod session;

use crate::session::Session;
use crate::protocol::{Package, Parsable};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tokio::net::TcpStream;
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use url::Url;


pub struct Connection{
    session: Session,
    socket: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
}

impl Connection{
    pub fn new() -> Self{
        Self{session: Session::new(), socket: None}
    }
    pub fn from_session(session: Session) -> Self{
        Self{session, socket: None}
    }
    async fn setup_session(&self) -> Result<(), ()>{
        Ok(())
    }
    pub fn get_session(&self) -> Session{ self.session.clone() }
    pub async fn set_session(&mut self, session: Session) -> Result<(), ()>{
        self.session = session;
        self.setup_session().await
    }
    pub async fn open(&mut self, url: String) -> Result<(), ()>{
        match Url::parse(&url){
            Ok(url) => {
                match connect_async(url).await{
                    Ok(conn) => {
                        self.socket = Some(conn.0);
                        self.setup_session().await
                    }
                    Err(_) => { Err(()) }
                }
            }
            Err(_) => { Err(()) }
        }
    }
    pub async fn close(&mut self){
        match self.socket.as_mut(){
            None => {}
            Some(we) => {
                match we.close(None).await{
                    Ok(_) => {}
                    Err(_) => {}
                }
            }
        }
    }
    pub async fn subscribe(&mut self, key: &str) -> Result<(), ()> {
        match self.session.sub(key.to_string()){
            None => { Ok(()) }
            Some(key) => {
                match self.socket.as_mut(){
                    None => {
                        self.session.unsub(key.to_string());
                        Err(())
                    }
                    Some(socket) => {
                        let res = socket.send(Message::Binary(
                            protocol::SubPackage{is_sub: true, int_key: key}.to_bytes()
                        )).await;
                        match res{
                            Ok(_) => { Ok(()) }
                            Err(_) => {
                                self.session.unsub(key.to_string());
                                Err(())
                            }
                        }
                    }
                }
            }
        }
    }
    pub async fn unsubscribe(&mut self, key: &str) -> Result<(), ()>{
        match self.session.unsub(key.to_string()){
            None => { Ok(()) }
            Some(key) => {
                match self.socket.as_mut(){
                    None => {
                        self.session.sub(key.to_string());
                        Err(())
                    }
                    Some(socket) => {
                        let res = socket.send(Message::Binary(
                            protocol::SubPackage{is_sub: false, int_key: key}.to_bytes()
                        )).await;
                        match res {
                            Ok(_) => { Ok(()) }
                            Err(_) => {
                                self.session.sub(key.to_string());
                                Err(())
                            }
                        }
                    }
                }
            }
        }
    }
    pub async fn set_node(&mut self, is_node: bool) -> Result<(), ()>{
        match self.session.set_a_node(is_node){
            None => { Ok(()) }
            Some(_) => {
                match self.socket.as_mut(){
                    None => {
                        self.session.set_a_node(!is_node);
                        Err(())
                    }
                    Some(socket) => {
                        let res = socket.send(Message::Binary(
                            protocol::NodePackage{is_node}.to_bytes()
                        )).await;
                        match res{
                            Ok(_) => { Ok(()) }
                            Err(_) => {
                                self.session.set_a_node(!is_node);
                                Err(())
                            }
                        }
                    }
                }
            }
        }
    }
    pub async fn next(&mut self) -> Result<protocol::Package, ()>{
        match self.socket.as_mut(){
            None => { Err(()) }
            Some(socket) => {
                loop{
                    match socket.next().await{
                        None => {
                            return Err(());
                        }
                        Some(msg) => {
                            match msg{
                                Ok(msg) => {
                                    if let Message::Binary(msg) = msg{
                                        if let Some(package) = protocol::Package::from_bytes(msg){
                                            return Ok(package);
                                        }
                                    }
                                }
                                Err(_) => {
                                    return Err(());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    pub async fn next_filtered(&mut self) -> Result<protocol::Package, ()>{
        loop{
            match self.next().await{
                Ok(msg) => {
                    match msg{
                        Package::Sub(sub) => {
                            if self.session.is_a_node(){
                                return Ok(protocol::Package::Sub(sub));
                            }
                        }
                        Package::Msg(msg) => {
                            if let Some(_) = self.session.is_sub(msg.str_key.clone()){
                                return Ok(protocol::Package::Msg(msg));
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => {
                    return Err(());
                }
            }
        }
    }
}