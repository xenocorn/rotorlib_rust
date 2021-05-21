use std::convert::{TryFrom, Into};
use md5;

#[cfg(test)]
mod subscribe_package_tests{
    use crate::protocol::SubscribePackage;
    use std::convert::TryFrom;

    #[test]
    fn test(){
        let reference_1 = SubscribePackage{
            is_sub: true,
            route_key: 42,
            msg_key: "SOME KEY".to_string()
        };
        let reference_2 = SubscribePackage{
            is_sub: false,
            route_key: 42,
            msg_key: "".to_string()
        };
        let result: Vec<u8> = reference_1.clone().into();
        match SubscribePackage::try_from(result){
            Ok(result) => {
                assert_eq!(reference_1, result)
            }
            Err(_) => {assert!(false, "Unexpected error")}
        }
        let result: Vec<u8> = reference_2.clone().into();
        match SubscribePackage::try_from(result){
            Ok(result) => {
                assert_eq!(reference_2, result)
            }
            Err(_) => {assert!(false, "Unexpected error")}
        }
    }
}

#[cfg(test)]
mod registration_package_tests{
    use crate::protocol::RegistrationPackage;
    use std::convert::TryFrom;

    #[test]
    fn test(){
        let reference = RegistrationPackage{ is_router: false };
        let result: Vec<u8> = reference.clone().into();
        match RegistrationPackage::try_from(result){
            Ok(result) => {
                assert_eq!(reference, result)
            }
            Err(_) => {assert!(false, "Unexpected error")}
        }
    }
}

#[cfg(test)]
mod message_package_tests{
    use crate::protocol::MessagePackage;
    use std::convert::TryFrom;

    #[test]
    fn test(){
        let reference_1 = MessagePackage{
            route_key: 42,
            msg_key: "SOME KEY".to_string(),
            payload: vec![1, 2, 3, 4, 5],
        };
        let reference_2 = MessagePackage{
            route_key: 42,
            msg_key: "".to_string(),
            payload: vec![],
        };
        let result: Vec<u8> = reference_1.clone().into();
        match MessagePackage::try_from(result){
            Ok(result) => {
                assert_eq!(reference_1, result)
            }
            Err(_) => {assert!(false, "Unexpected error")}
        }
        let result: Vec<u8> = reference_2.clone().into();
        match MessagePackage::try_from(result){
            Ok(result) => {
                assert_eq!(reference_2, result)
            }
            Err(_) => {assert!(false, "Unexpected error")}
        }
    }
}

#[cfg(test)]
mod package_tests{
    use crate::protocol::{Package, SubscribePackage, RegistrationPackage, MessagePackage};
    use std::convert::TryFrom;

    #[test]
    fn test_sub(){
        let reference_1 = Package::Sub(SubscribePackage{
            is_sub: true,
            route_key: 42,
            msg_key: "SOME KEY".to_string()
        });
        let reference_2 = Package::Sub(SubscribePackage{
            is_sub: false,
            route_key: 42,
            msg_key: "".to_string()
        });
        let result: Vec<u8> = reference_1.clone().into();
        match Package::try_from(result){
            Ok(result) => {
                assert_eq!(reference_1, result)
            }
            Err(_) => {assert!(false, "Unexpected error")}
        }
        let result: Vec<u8> = reference_2.clone().into();
        match Package::try_from(result){
            Ok(result) => {
                assert_eq!(reference_2, result)
            }
            Err(_) => {assert!(false, "Unexpected error")}
        }
    }
    #[test]
    fn test_reg(){
        let reference = Package::Reg(RegistrationPackage{ is_router: false });
        let result: Vec<u8> = reference.clone().into();
        match Package::try_from(result){
            Ok(result) => {
                assert_eq!(reference, result)
            }
            Err(_) => {assert!(false, "Unexpected error")}
        }
    }
    #[test]
    fn test_msg(){
        let reference_1 = Package::Msg(MessagePackage{
            route_key: 42,
            msg_key: "SOME KEY".to_string(),
            payload: vec![1, 2, 3, 4, 5],
        });
        let reference_2 = Package::Msg(MessagePackage{
            route_key: 42,
            msg_key: "".to_string(),
            payload: vec![],
        });
        let result: Vec<u8> = reference_1.clone().into();
        match Package::try_from(result){
            Ok(result) => {
                assert_eq!(reference_1, result)
            }
            Err(_) => {assert!(false, "Unexpected error")}
        }
        let result: Vec<u8> = reference_2.clone().into();
        match Package::try_from(result){
            Ok(result) => {
                assert_eq!(reference_2, result)
            }
            Err(_) => {assert!(false, "Unexpected error")}
        }
    }
}

pub type Key = u64;

pub fn get_route_key(msg_key: &String) -> Key{
    let digest = md5::compute(msg_key.clone()).0;
    let mut dst = [0u8; 8];
    dst.clone_from_slice(&digest[0..8]);
    Key::from_be_bytes(dst)
}

#[derive(Debug, PartialEq, Clone)]
pub struct SubscribePackage {
    pub is_sub: bool,
    pub route_key: Key,
    pub msg_key: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RegistrationPackage{
    pub is_router: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MessagePackage {
    pub route_key: Key,
    pub msg_key: String,
    pub payload: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Package{
    Sub(SubscribePackage),
    Reg(RegistrationPackage),
    Msg(MessagePackage),
}

impl Into<Vec<u8>> for SubscribePackage {
    fn into(self) -> Vec<u8> {
        let header: u8 = match self.is_sub{
            true => {0b01100000}
            false => {0b01000000}
        };
        let mut ret = vec![header];
        ret.append(&mut self.route_key.to_be_bytes().to_vec());
        ret.append(&mut self.msg_key.into_bytes());
        ret
    }
}

impl TryFrom<Vec<u8>> for SubscribePackage {
    type Error = ();

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.len() < 9 { return Err(()) }
        let is_sub = (bytes[0] >> 5) & 0b00000001;
        let is_sub = if is_sub == 0 {false} else {true};
        let mut dst = [0u8; 8];
        dst.clone_from_slice(&bytes[1..9]);
        let route_key = Key::from_be_bytes(dst);
        match String::from_utf8(bytes[9..].to_owned()){
            Ok(msg_key) => {
                Ok(Self{is_sub, route_key, msg_key})
            }
            Err(_) => { Err(()) }
        }
    }
}

impl Into<Vec<u8>> for RegistrationPackage {
    fn into(self) -> Vec<u8> {
        match self.is_router {
            true => {vec![0b10100000]}
            false => {vec![0b10000000]}
        }
    }
}

impl TryFrom<Vec<u8>> for RegistrationPackage {
    type Error = ();

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let is_router = (bytes[0] >> 5) & 0b00000001;
        let is_router = if is_router == 0 {false} else {true};
        Ok(Self{ is_router })
    }
}

impl Into<Vec<u8>> for MessagePackage {
    fn into(self) -> Vec<u8> {
        let mut ret: Vec<u8> = vec![0b00000000];
        ret.append(&mut self.route_key.to_be_bytes().to_vec());
        ret.append(&mut self.msg_key.clone().into_bytes());
        ret.push(0); // separator
        ret.append(&mut self.payload.clone());
        ret
    }
}

impl TryFrom<Vec<u8>> for MessagePackage {
    type Error = ();

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.len() < 9 { return Err(()) }
        let mut dst = [0u8; 8];
        dst.clone_from_slice(&bytes[1..9]);
        let route_key = Key::from_be_bytes(dst);
        let mut str_ends: usize = 5;
        for (nom, char) in bytes[9..].iter().enumerate(){
            str_ends = nom+9;
            if *char == 0{ break }
        }
        match String::from_utf8(bytes[9..str_ends].to_owned()){
            Ok(msg_key) => {
                let payload = bytes[str_ends+1..].to_vec();
                Ok(Self{ route_key, msg_key, payload})
            }
            Err(_) => { Err(()) }
        }
    }
}

impl Into<Vec<u8>> for Package {
    fn into(self) -> Vec<u8> {
        match self{
            Package::Msg(msg) => {msg.into()}
            Package::Sub(sub) => {sub.into()}
            Package::Reg(node) => {node.into()}
        }
    }
}

impl TryFrom<Vec<u8>> for Package {
    type Error = ();

    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        if bytes.len() < 1 { return Err(()) }
        let type_ = bytes[0] >> 6;
        match type_{
            0 => {
                match MessagePackage::try_from(bytes){
                    Err(_) => { Err(()) }
                    Ok(msg) => { Ok(Self::Msg(msg)) }
                }
            }
            1 => {
                match SubscribePackage::try_from(bytes){
                    Err(_) => { Err(()) }
                    Ok(sub) => { Ok(Self::Sub(sub)) }
                }
            }
            2 => {
                match RegistrationPackage::try_from(bytes){
                    Err(_) => { Err(()) }
                    Ok(node) => { Ok(Self::Reg(node)) }
                }
            }
            _ => { Err(()) }
        }
    }
}
