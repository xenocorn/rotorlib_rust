use crate::protocol::{Package, SubscribePackage, RegistrationPackage};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};
use tokio::net::TcpStream;
use url::Url;
use tokio_tungstenite::tungstenite::{Error as T_Error, Message};
use futures::{SinkExt, StreamExt};
use std::convert::TryFrom;
use crate::session::Session;
use tokio::time::{sleep, Duration};


pub struct LightClient{
    socket: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl LightClient{
    pub async fn open(url: Url) -> Result<Self, T_Error>{
        match connect_async(url).await{
            Ok(socket) => {
                Ok(Self::new(socket.0))
            }
            Err(err) => { Err(err) }
        }
    }
    pub fn new(socket: WebSocketStream<MaybeTlsStream<TcpStream>>) -> Self{ Self{socket} }
    pub async fn close(&mut self) -> Result<(), T_Error> {
        self.socket.close(None).await
    }
    pub async fn send(&mut self, package: Package) -> Result<(), T_Error> {
        self.socket.send(Message::Binary(package.into())).await
    }
    pub async fn recv(&mut self) -> Result<Package, T_Error>{
        loop {
            match self.socket.next().await{
                None => { return Err(T_Error::ConnectionClosed) }
                Some(res) => {
                    match res{
                        Ok(msg) => {
                            if let Message::Binary(bin) = msg{
                                if let Ok(package) = Package::try_from(bin){
                                    return Ok(package)
                                }
                            }
                        }
                        Err(err) => { return Err(err) }
                    }
                }
            }
        }
    }
}

fn increment_with_limit(mut counter: u64, step: u64, limit: u64) -> Result<u64, ()>{
    counter += step;
    if limit != 0 {
        if counter >= limit{ return Err(()) }
    }
    Ok(counter)
}

pub struct Client{
    url: Url,
    max_reconnects: u64,
    delay_step: u64,
    connection: Option<LightClient>,
    session: Session,
}

impl Client{
    pub fn new(session: Session, url: Url, max_reconnects: u64, delay_step: u64) -> Self{
        Self{url, max_reconnects, delay_step, connection: None, session}
    }
    pub fn get_session(&self) -> Session{ self.session.clone() }
    async fn setup_connection(&mut self) -> Result<(), ()>{
        match self.connection.as_mut(){
            None => { return Err(()) }
            Some(conn) => {
                if self.session.is_a_router(){
                    let resp = conn.send(Package::Reg(RegistrationPackage{is_router: true})).await;
                    if let Err(_) = resp{ return Err(()) }
                }
                for key in self.session.get_subscriptions(){
                    let resp = conn.send(Package::Sub(SubscribePackage{
                        is_sub: true,
                        key,
                    })).await;
                    if let Err(_) = resp{ return Err(()) }
                }
            }
        }
        Ok(())
    }
    async fn open(&mut self) -> Result<u64, ()>{
        let mut reconnects: u64 = 0;
        loop {
            match self.connection.as_mut(){
                None => {
                    let delay = reconnects * self.delay_step;
                    sleep(Duration::from_millis(delay)).await;
                    if let Ok(conn) =  LightClient::open(self.url.clone()).await{
                        self.connection = Some(conn);
                        match self.setup_connection().await{
                            Ok(_) => { return Ok(reconnects)}
                            Err(_) => { self.close().await; }
                        }
                    }
                }
                Some(_) => {
                    self.close().await;
                    continue;
                }
            }
            match increment_with_limit(reconnects, 0, self.max_reconnects){
                Ok(r) => { reconnects = r; }
                Err(_) => { return Err(()) }
            }
        }
    }
    pub async fn close(&mut self){
        if let Some(conn) = self.connection.as_mut(){
            if let Err(_) = conn.close().await{}
        }
        self.connection = None;
    }
    pub async fn send(&mut self, package: Package) -> Result<(), ()>{
        let mut reconnects: u64 = 0;
        let mut step;
        loop{
            step = 1;
            match self.connection.as_mut(){
                None => {
                    match self.open().await{
                        Ok(r) => { step += r; }
                        Err(_) => { return Err(()) }
                    }
                }
                Some(connection) => {
                    match connection.send(package.clone()).await{
                        Ok(_) => { return Ok(()) }
                        Err(_) => { self.close().await }
                    }
                }
            }
            match increment_with_limit(reconnects, step, self.max_reconnects){
                Ok(r) => { reconnects = r; }
                Err(_) => { return Err(()) }
            }
        }
    }
    pub async fn recv(&mut self) -> Result<Package, ()>{
        let mut reconnects: u64 = 0;
        let mut step;
        loop{
            step = 1;
            match self.connection.as_mut(){
                None => {
                    match self.open().await{
                        Ok(r) => { step += r; }
                        Err(_) => { return Err(()) }
                    }
                }
                Some(connection) => {
                    match connection.recv().await{
                        Ok(package) => { return Ok(package) }
                        Err(_) => { self.close().await }
                    }
                }
            }
            match increment_with_limit(reconnects, step, self.max_reconnects){
                Ok(r) => { reconnects = r; }
                Err(_) => { return Err(()) }
            }
        }
    }
    pub async fn subscribe(&mut self, key: String) -> Result<(), ()>{
        match self.session.sub(key.clone()){
            true => {
                let ret = self.send(Package::Sub(SubscribePackage{
                    is_sub: true,
                    key: key.clone(),
                })).await;
                match ret{
                    Ok(_) => { Ok(()) }
                    Err(_) => {
                        self.session.unsub(key);
                        Err(())
                    }
                }
            }
            false => { Ok(()) }
        }
    }
    pub async fn unsubscribe(&mut self, key: String) -> Result<(), ()>{
        match self.session.unsub(key.clone()){
            true => {
                let ret = self.send(Package::Sub(SubscribePackage{
                    is_sub: false,
                    key: key.clone(),
                })).await;
                match ret{
                    Ok(_) => { Ok(()) }
                    Err(_) => {
                        self.session.sub(key);
                        Err(())
                    }
                }
            }
            false => { Ok(()) }
        }
    }
    pub async fn reg(&mut self, is_a_router: bool) -> Result<(), ()>{
        if self.session.is_a_router() != is_a_router{
            self.session.set_a_router(is_a_router);
            let ret = self.send(Package::Reg(RegistrationPackage{is_router: is_a_router})).await;
            if let Err(_) = ret{
                self.session.set_a_router(!is_a_router);
            }
            return ret
        }
        Ok(())
    }
    pub async fn next(&mut self) -> Result<Package, ()>{
        loop {
            match self.recv().await{
                Ok(package) => {
                    match package{
                        Package::Sub(sub) => {
                            if self.session.is_a_router(){
                                return Ok(Package::Sub(sub))
                            }
                        }
                        Package::Reg(reg) => {
                            return Ok(Package::Reg(reg))
                        }
                        Package::Msg(msg) => {
                            if self.session.is_sub(&msg.key){
                                return Ok(Package::Msg(msg))
                            }
                        }
                    }
                }
                Err(_) => { return Err(()) }
            }
        }
    }
}
