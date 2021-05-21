#[derive(Debug, Clone)]
pub struct Session{
    is_self_a_router: bool,
    subscriptions: Vec<String>,
}

impl Session{
    pub fn new() -> Self{
        Self{ is_self_a_router: false, subscriptions: Vec::new()}
    }
    pub fn set_a_router(&mut self, is_a_router: bool) -> Option<()>{
        match is_a_router != self.is_self_a_router {
            true => {
                self.is_self_a_router = is_a_router;
                Some(())
            }
            false => { None }
        }
    }
    pub fn is_a_router(&self) -> bool{ self.is_self_a_router }
    pub fn clear_subscriptions(&mut self) -> &Self{
        self.subscriptions = Vec::new();
        self
    }
    pub fn sub(&mut self, key: String) -> bool{
        if self.subscriptions.contains(&key){
            return false;
        }
        self.subscriptions.push(key);
        self.subscriptions.sort();
        return true;
    }
    pub fn unsub(&mut self, key: String) -> bool{
        match self.subscriptions.binary_search(&key){
            Ok(position) => {
                self.subscriptions.remove(position);
                true
            }
            Err(_) => { false }
        }
    }
    pub fn is_sub(&self, key: &String) -> bool{
        self.subscriptions.contains(&key)
    }
    pub fn get_subscriptions(&self) -> Vec<String>{
        self.subscriptions.clone()
    }
}
