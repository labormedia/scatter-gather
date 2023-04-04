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

#[derive(Debug)]
pub struct Route(pub PeerId, pub Distance);

impl<'a> PartialEq for Route {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl<'a> Eq for Route {}

impl<'a> PartialOrd for Route {
    fn partial_cmp(&self, other: &Self) -> Option<core_::cmp::Ordering> {
        let ord = 
            if self.1 < other.1 {
                std::cmp::Ordering::Less
            } else if self.1 == other.1 {
                std::cmp::Ordering::Equal
            } else {
                std::cmp::Ordering::Greater
            };
        Some(ord)
    }
}

impl<'a> Ord for Route {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1.cmp(&other.1)
    }
}

pub struct Router  {
    peer_id: PeerId,
    pub peer_list: Vec<Box<PeerId>>,
    key: Key<PeerId>
}

impl<'a> Router {
    pub fn from(peer_id: PeerId, mut peer_list: Vec<Box<PeerId>>) -> Self {
        peer_list.sort();
        Self { 
            peer_id, 
            peer_list, 
            key: Key::from(peer_id)
        }
    }
    pub fn sort(mut self) {
        self.peer_list.sort();
    }
    pub fn closest(self, peer_id: PeerId) -> Route {
        let search_key = Key::from(peer_id);
        self
            .peer_list
            .iter()
            .fold( Route(self.peer_id, Distance::MAX), |min, boxed_router| {
                let key_other = Key::from(**boxed_router);
                let distance = search_key.distance(&key_other);
                if min.1 <= distance {
                    min
                } else {
                    Route(**boxed_router, distance)
                }
            }   )
    }
    pub fn k_closest(self, peer_id: PeerId, k: usize) -> Vec<Route> {
        let search_key = Key::from(peer_id);
        let mut routes: Vec<Route> = self
            .peer_list
            .iter()
            .map( |boxed_router| {
                let key_other = Key::from(**boxed_router);
                Route(**boxed_router, search_key.distance(&key_other))
            })
            .collect();
        routes.dedup();
        routes.sort();
        routes
            .into_iter()
            .take(k)
            .collect()
            // .take(k)
    }
}