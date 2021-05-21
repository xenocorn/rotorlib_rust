use crate::protocol::{Package, MessagePackage};
use url::{Url};
use hyper::{Request, Body, Client};
use hyper::client::connect::HttpConnector;
use hyper_tls::HttpsConnector;
use http::uri::{Uri, InvalidUri};
use std::convert::TryFrom;

fn url_to_uri(url: Url) -> Result<Uri, InvalidUri> {
    Uri::try_from(url.as_str())
}

pub async fn send_msg(node_url: Url, msg: MessagePackage) -> Result<(), ()>{
    let body: Vec<u8> = Package::Msg(msg).into();
    let https = HttpsConnector::new();
    let client: Client<HttpsConnector<HttpConnector>> = Client::builder().build::<_, hyper::Body>(https);
    match node_url.join("send"){
        Ok(url) => {
            match url_to_uri(url){
                Ok(uri) => {
                    let req = Request::post(uri).body(Body::from(body));
                    match req{
                        Ok(req) => {
                            match client.request(req).await{
                                Ok(resp) => {
                                    match resp.status() == 200{
                                        true => { Ok(()) }
                                        false => { Err(()) }
                                    }
                                }
                                Err(_) => { Err(()) }
                            }
                        }
                        Err(_) => { Err(()) }
                    }
                }
                Err(_) => { Err(()) }
            }
        }
        Err(_) => { Err(()) }
    }
}
