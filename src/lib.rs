use scatter_gather_models::peer_id::PeerId;
use scatter_gather_models::xor::{
    Distance,
    Key
};
// use rand::prelude::SliceRandom;
use rand::prelude::IteratorRandom;
use std::collections::HashMap;
use uint::*;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}
pub struct DHT {
    pub routes: HashMap<PeerId, Vec<Box<PeerId>>>
}

impl DHT {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new()
        }
    }
    pub fn routing(mut self, collection: Vec<PeerId>, router_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        let new_collection = collection
                .iter()
                .fold( HashMap::new(),
                    | mut a: HashMap<PeerId, Vec<Box<PeerId>>>, peer_id: &PeerId|
                    {
                        let peer_list = collection
                            .iter()
                            .map(|x| {
                                Box::new(*x)
                            })
                            .choose_multiple(&mut rand::thread_rng(), router_size);

                        a.insert(*peer_id, peer_list);
                        
                        a
                    }
                );
        self.routes = new_collection;
        Ok(self)
    }
}

pub struct Router  {
    peer_id: PeerId,
    pub peer_list: Vec<Box<PeerId>>,
    key: Key<PeerId>
}

impl<'a> PartialEq for Router {
    fn eq(&self, other: &Self) -> bool {
        self.peer_id == other.peer_id
    }
}

impl<'a> Eq for Router {}

impl<'a> PartialOrd for Router {
    fn partial_cmp(&self, other: &Self) -> Option<core_::cmp::Ordering> {
        let ord = 
            if self.peer_id < other.peer_id {
                std::cmp::Ordering::Less
            } else if self.peer_id == other.peer_id {
                std::cmp::Ordering::Equal
            } else {
                std::cmp::Ordering::Greater
            };
        Some(ord)
    }
}

impl<'a> Ord for Router {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.peer_id.cmp(&other.peer_id)
    }
}

impl<'a> Router {
    pub fn from(peer_id: PeerId, peer_list: Vec<Box<PeerId>>) -> Self {
        Self { 
            peer_id, 
            peer_list, 
            key: Key::from(peer_id)
        }
    }
    pub fn sort(mut self) {
        self.peer_list.sort();
    }
    pub fn closest(self, peer_id: PeerId) -> (PeerId, Distance) {
        let search_key = Key::from(peer_id);
        self
            .peer_list
            .iter()
            .fold( (self.peer_id, Distance::MAX), |min, boxed_router| {
                let key_other = Key::from(**boxed_router);
                let distance = search_key.distance(&key_other);
                if min.1 <= distance {
                    min
                } else {
                    (**boxed_router, distance)
                }
            }   )
    }
    pub fn k_closest(mut self, peer_id: PeerId, k: usize) -> Vec<(PeerId, Distance)> {
        let search_key = Key::from(peer_id);
        self.peer_list.sort();
        self
            .peer_list
            .iter()
            .take(k)
            .fold( Vec::new(), |mut a, boxed_router| {
                let key_other = Key::from(**boxed_router);
                a.push((**boxed_router, search_key.distance(&key_other)));
                a
            })
    }
}