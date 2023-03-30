use scatter_gather_models::peer_id::PeerId;
use scatter_gather_models::xor::{
    Distance,
    Key
};
use rand::prelude::SliceRandom;
use std::collections::HashMap;
use uint::*;

construct_uint! {
    /// 256-bit unsigned integer.
    pub struct U256(4);
}

pub fn routing(network_size: usize, router_size: usize) -> Result<(Vec<PeerId>, HashMap<PeerId, Vec<PeerId>>), Box<dyn std::error::Error>> {
    let mut collection: Vec<PeerId> = Vec::new();
    for _i in 0..network_size {
        collection.push(
                PeerId::random()
        );
    }
    let cloned_collection = collection.clone();
    let new_collection = collection.
        into_iter()
        .fold( HashMap::new(),
            |mut a: HashMap<PeerId, Vec<PeerId>>, x: PeerId|
            {
                a.insert(x, cloned_collection
                    .choose_multiple(&mut rand::thread_rng(), router_size)
                    .cloned()
                    .collect());
                a
            }
            
        );
    Ok((cloned_collection, new_collection))
}

pub struct Router<'a> {
    peer_id: PeerId,
    peer_list: &'a mut Vec<PeerId>,
    key: Key<PeerId>
}

impl<'a> Router<'a> {
    pub fn from(peer_id: PeerId, peer_list: &'a mut Vec<PeerId>) -> Self {
        Self { 
            peer_id, 
            peer_list, 
            key: Key::from(peer_id)
        }
    }
    pub fn sort(self) {
        self.peer_list.sort();
    }
    pub fn closest(self, peer_id: PeerId) -> (PeerId, Distance) {
        let search_key = Key::from(peer_id);
        self
            .peer_list
            .iter()
            .fold( (self.peer_id, Distance::MAX), |min, x| {
                let key_other = Key::from(*x);
                let distance = search_key.distance(&key_other);
                if min.1 <= distance {
                    min
                } else {
                    (*x, distance)
                }
            }   )
    }
}