use md5;

#[cfg(test)]
mod sub_package_tests {
    use crate::protocol::{SubPackage, Parsable};
    #[test]
    fn to_bytes(){
        let int_key: u32 = 42;
        let mut reference: Vec<u8> = vec![0b00100000];
        reference.append(&mut int_key.to_be_bytes().to_vec());
        let bytes = SubPackage{is_sub: true, int_key}.to_bytes();
        assert_eq!(bytes, reference);
    }

    #[test]
    fn from_bytes(){
        let int_key: u32 = 42;
        let reference = SubPackage{is_sub: true, int_key};
        let mut bytes: Vec<u8> = vec![0b00100000];
        bytes.append(&mut int_key.to_be_bytes().to_vec());
        let test = SubPackage::from_bytes(bytes).unwrap();
        assert_eq!(test, reference);
    }

    #[test]
    fn complex(){
        let int_key: u32 = 42;
        let package = SubPackage{is_sub: true, int_key};
        assert_eq!(package, SubPackage::from_bytes(package.to_bytes()).unwrap())
    }
}

#[cfg(test)]
mod node_package_tests {
    use crate::protocol::{Parsable, NodePackage};

    #[test]
    fn to_bytes(){
        let reference = vec![0b10100000];
        let package = NodePackage{is_node: true};
        assert_eq!(reference, package.to_bytes());
        let reference = vec![0b10000000];
        let package = NodePackage{is_node: false};
        assert_eq!(reference, package.to_bytes());
    }

    #[test]
    fn from_bytes(){
        let reference = NodePackage{is_node: true};
        let bytes = vec![0b10100000];
        assert_eq!(reference, NodePackage::from_bytes(bytes).unwrap());
        let reference = NodePackage{is_node: false};
        let bytes = vec![0b10000000];
        assert_eq!(reference, NodePackage::from_bytes(bytes).unwrap());
    }

    #[test]
    fn complex(){
        let package = NodePackage{is_node: true};
        assert_eq!(package, NodePackage::from_bytes(package.to_bytes()).unwrap());
        let package = NodePackage{is_node: false};
        assert_eq!(package, NodePackage::from_bytes(package.to_bytes()).unwrap());
    }
}

#[cfg(test)]
mod msg_package_tests {
    use crate::protocol::{Parsable, MsgPackage};

    /*
    #[test]
    fn to_bytes(){}

    #[test]
    fn from_bytes(){}
    */

    #[test]
    fn complex(){
        let package = MsgPackage{int_key: 42, str_key: String::from("abcdef123456"), payload: vec![0, 1, 2, 3]};
        assert_eq!(package, MsgPackage::from_bytes(package.to_bytes()).unwrap());
    }
}

#[cfg(test)]
mod package_tests {
    use crate::protocol::{Parsable, Package, SubPackage, NodePackage, MsgPackage};

    #[test]
    fn sub(){
        let package = Package::Sub(SubPackage{is_sub: true, int_key: 42});
        assert_eq!(package, Package::from_bytes(package.to_bytes()).unwrap());
    }

    #[test]
    fn node(){
        let package = Package::Node(NodePackage{is_node: true});
        assert_eq!(package, Package::from_bytes(package.to_bytes()).unwrap());
    }

    #[test]
    fn msg(){
        let package = Package::Msg(MsgPackage{int_key: 42, str_key: String::from("abcdef12345"), payload: vec![1,2,3,4]});
        assert_eq!(package, Package::from_bytes(package.to_bytes()).unwrap());
    }
}

pub type Key = u32;

pub trait Parsable{
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> where Self: Sized;
    fn to_bytes(&self) -> Vec<u8>;
}

#[derive(Debug, PartialEq)]
pub struct SubPackage{
    pub is_sub: bool,
    pub int_key: Key,
}

#[derive(Debug, PartialEq)]
pub struct NodePackage{
    pub is_node: bool,
}

#[derive(Debug, PartialEq)]
pub struct MsgPackage{
    pub int_key: Key,
    pub str_key: String,
    pub payload: Vec<u8>,
}

impl Parsable for SubPackage{
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() < 5 { return None }
        let is_sub = (bytes[0] >> 5) & 0b00000001;
        let is_sub = if is_sub == 0 {false} else {true};
        let mut dst = [0u8; 4];
        dst.clone_from_slice(&bytes[1..5]);
        let int_key = Key::from_be_bytes(dst);
        Some(Self{is_sub, int_key})
    }

    fn to_bytes(&self) -> Vec<u8> {
        let header: u8 = match self.is_sub{
            true => {0b00100000}
            false => {0b00000000}
        };
        let mut ret = vec![header];
        ret.append(&mut self.int_key.clone().to_be_bytes().to_vec());
        ret
    }
}

impl Parsable for NodePackage{
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        let is_node = (bytes[0] >> 5) & 0b00000001;
        let is_node = if is_node == 0 {false} else {true};
        Some(Self{is_node})
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self.is_node{
            true => {vec![0b10100000]}
            false => {vec![0b10000000]}
        }
    }
}

impl Parsable for MsgPackage{
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() < 6 { return None }
        let mut dst = [0u8; 4];
        dst.clone_from_slice(&bytes[1..5]);
        let int_key = Key::from_be_bytes(dst);
        let mut str_ends: usize = 5;
        for (nom, char) in bytes[5..].iter().enumerate(){
            str_ends = nom+5;
            if *char == 0{ break }
        }
        match String::from_utf8(bytes[5..str_ends].to_owned()){
            Ok(str_key) => {
                let payload = bytes[str_ends+1..].to_vec();
                Some(Self{int_key, str_key, payload})
            }
            Err(_) => { None }
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut ret: Vec<u8> = vec![0b01000000];
        ret.append(&mut self.int_key.to_be_bytes().to_vec());
        ret.append(&mut self.str_key.clone().into_bytes());
        ret.push(0); // separator
        ret.append(&mut self.payload.clone());
        ret
    }
}

#[derive(Debug, PartialEq)]
pub enum Package{
    Sub(SubPackage),
    Node(NodePackage),
    Msg(MsgPackage),
}

impl Parsable for Package{
    fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        if bytes.len() < 1 { return None }
        let type_ = bytes[0] >> 6;
        match type_{
            0 => {
                match SubPackage::from_bytes(bytes){
                    None => { None }
                    Some(sub) => { Some(Self::Sub(sub)) }
                }
            }
            1 => {
                match MsgPackage::from_bytes(bytes){
                    None => { None }
                    Some(msg) => { Some(Self::Msg(msg)) }
                }
            }
            2 => {
                match NodePackage::from_bytes(bytes){
                    None => { None }
                    Some(node) => { Some(Self::Node(node)) }
                }
            }
            _ => { None }
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        match self{
            Package::Sub(sub) => {sub.to_bytes()}
            Package::Node(node) => {node.to_bytes()}
            Package::Msg(msg) => {msg.to_bytes()}
        }
    }
}

pub fn hash(s: &String) -> Key{
    let digest = md5::compute(s.clone()).0;
    let mut dst = [0u8; 4];
    dst.clone_from_slice(&digest[0..4]);
    Key::from_be_bytes(dst)
}
