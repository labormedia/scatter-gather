use scatter_gather_models::peer_id::PeerId;
use scatter_gather_models::xor::{
    Distance,
    Key
};
use rayon::prelude::*;
// use rand::prelude::SliceRandom;
use rand::prelude::SliceRandom;
use std::collections::HashMap;
use uint::*;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}
pub struct DHT {
    pub routes: HashMap<PeerId, Vec<Route>>
}

impl DHT {
    pub fn new() -> Self {
        Self {
            routes: HashMap::new()
        }
    }
    pub fn routing(mut self, collection: Vec<PeerId>, router_size: usize) -> Result<Self, Box<dyn std::error::Error>> {
        self.routes = collection
                .iter()
                .map(|peer_id| {
                    let rng = &mut rand::thread_rng();
                    let router_list: Vec<PeerId> = collection
                        .choose_multiple(rng, router_size)
                        .par_bridge()
                        .into_par_iter()
                        .map(|x| {
                            *x
                        })
                        .collect::<Vec<PeerId>>();
                    (*peer_id, router_list)
                })
                .fold( HashMap::new(),
                    | mut a: HashMap<PeerId, Vec<Route>>, peer_id: (PeerId, Vec<PeerId>)|
                    {
                        let router_id = peer_id.0;
                        let router_key = Key::from(router_id);
                        let rng = &mut rand::thread_rng();
                        let candidate_list = collection
                            .choose_multiple(rng, router_size)
                            .par_bridge()
                            .into_par_iter()
                            .map(|peer_id| {
                                Route(*peer_id, router_key.distance(&Key::from(*peer_id)))
                            })
                            .collect();
                        let router_list = Router::from(router_id, candidate_list)
                            .k_closest(router_id, 20);
                        a.insert(router_id, router_list);
                        a
                    }
                )
                ;
        Ok(self)
    }
}

#[cfg(test)]
mod test_router {
    #[test]
    fn test_routes_generation() {
        use super::*;
        const ROUTER_SIZE: usize = 13;
        const NETWORK_SIZE: usize = 10_000;

        let mut collection: Vec<PeerId> = Vec::new();
        for _i in 0..NETWORK_SIZE {
            collection.push(
                    PeerId::random()
            );
        }

        let mut rng = rand::thread_rng();
        let random_id = collection.choose(&mut rng).expect("No PeerId.").clone();

        let dht = DHT::new().routing(collection, ROUTER_SIZE).expect("Cannot generate routing.");
        let routes: Vec<Route> = dht.routes.get(&random_id).expect("Could not find routes.").clone();
        let _assert = routes
            .into_iter()
            .map(|x| {
                let key = Key::from(random_id);
                assert!(key.distance(&Key::from(x.0)) == x.1 );
                x
            })
            .collect::<Vec<Route>>()
            ;
    }
}

#[derive(Debug, Clone)]
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
    pub peer_list: Vec<Route>,
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
    pub fn from(router_id: PeerId, peer_list: Vec<Route>) -> Self {
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
                let key_other = Key::from(router_id.0);
                let distance = search_key.distance(&key_other);
                if min.1 <= distance {
                    min
                } else {
                    Route(router_id.0, distance)
                }
            }   )
    }
    pub fn k_closest(self, peer_id: PeerId, k: usize) -> Vec<Route> {
        let search_key = Key::from(peer_id);
        let mut routes: Vec<Route> = self
            .peer_list
            .par_iter()
            .map( |router_id| {
                let key_other = Key::from(router_id.0);
                Route(router_id.0, search_key.distance(&key_other))
            })
            .collect();
        routes.par_sort();
        routes.dedup();
        routes
            .into_iter()
            .take(k)
            .collect()
    }
}