use scatter_gather_models::peer_id::PeerId;
use scatter_gather_models::xor::{
    Distance,
    Key
};
use rayon::prelude::*;
// use rand::prelude::SliceRandom;
use rand::prelude::{
    SliceRandom,
    IteratorRandom
};
use std::{
    collections::HashMap,
    sync::{
        Arc,
        Mutex
    }   
};
use uint::*;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}
pub struct DHT {
    pub routes: HashMap<PeerId, Vec<PeerId>>
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
                .map(|peer_id| {
                    let router_id = peer_id;
                    let rng = &mut rand::thread_rng();
                    let router_list: Vec<PeerId> = collection
                        .choose_multiple(rng, router_size)
                        .into_iter()
                        .map(|x| {
                            x.clone()
                        })
                        .collect();
                    (*peer_id, router_list)
                })
                .map(|x| {
                    x
                })
                .fold( HashMap::new(),
                    | mut a: HashMap<PeerId, Vec<PeerId>>, peer_id: (PeerId, Vec<PeerId>)|
                    {
                        let router_id = peer_id;
                        let rng = &mut rand::thread_rng();
                        let router_list = collection
                            .choose_multiple(rng, router_size)
                            .into_iter()
                            .map(|x| {
                                *x
                            })
                            .collect();

                        a.insert(router_id.0, router_list);
                        
                        a
                    }
                )
                // .collect()
                ;
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
    router_id: PeerId,
    pub peer_list: Vec<PeerId>,
    key: Key<PeerId>
}

impl<'a> PartialEq for Router {
    fn eq(&self, other: &Self) -> bool {
        self.router_id == other.router_id
    }
}

impl<'a> Eq for Router {}

impl<'a> PartialOrd for Router {
    fn partial_cmp(&self, other: &Self) -> Option<core_::cmp::Ordering> {
        let ord = 
            if self.router_id < other.router_id {
                std::cmp::Ordering::Less
            } else if self.router_id == other.router_id {
                std::cmp::Ordering::Equal
            } else {
                std::cmp::Ordering::Greater
            };
        Some(ord)
    }
}

impl<'a> Ord for Router {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.router_id.cmp(&other.router_id)
    }
}

impl<'a> Router {
    pub fn from(router_id: PeerId, mut peer_list: Vec<PeerId>) -> Self {
        peer_list.sort();
        Self { 
            router_id, 
            peer_list, 
            key: Key::from(router_id)
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
            .fold( Route(self.router_id, Distance::MAX), |min, router_id| {
                let key_other = Key::from(*router_id);
                let distance = search_key.distance(&key_other);
                if min.1 <= distance {
                    min
                } else {
                    Route(*router_id, distance)
                }
            }   )
    }
    pub fn k_closest(self, peer_id: PeerId, k: usize) -> Vec<Route> {
        let search_key = Key::from(peer_id);
        let mut routes: Vec<Route> = self
            .peer_list
            .par_iter()
            .map( |router_id| {
                let key_other = Key::from(*router_id);
                Route(*router_id, search_key.distance(&key_other))
            })
            .collect();
        routes.dedup();
        routes.par_sort();
        routes
            .into_par_iter()
            .take(k)
            .collect()
    }
}