use scatter_gather_models::peer_id::PeerId;
use scatter_gather_models::xor::{
    Distance,
    Key
};
use rand::prelude::SliceRandom;
use std::collections::HashMap;

pub fn routing(size: usize) -> Result<(Vec<PeerId>, HashMap<PeerId, Vec<PeerId>>), Box<dyn std::error::Error>> {
    let mut collection: Vec<PeerId> = Vec::new();
    for _i in 0..size {
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
                    .choose_multiple(&mut rand::thread_rng(), 17)
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
            .fold( (self.peer_id, Distance::default()), |min, x| {
                let key_other = Key::from(*x);
                let acc = if min.1 == Distance::default() {
                    (*x, search_key.distance(&key_other) )
                } else {
                    let new_distance = search_key.distance(&key_other);
                    println!("Compare : {:?}", new_distance);
                    if min.1 < new_distance {
                        min
                    } else {
                        (*x, new_distance)
                    }
                };
                acc

            }   )
    }
}