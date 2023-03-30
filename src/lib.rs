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

struct Router((PeerId, Vec<PeerId>));

impl Router {
    fn from(tuple : (PeerId, Vec<PeerId>)) -> Self {
        Self(tuple)
    }
    fn sort(mut self) {
        self.0.1.sort();
    }
    fn closest(mut self, peer_id: PeerId) -> PeerId {
        let distance = self
            .0.1
            .iter()
            .fold( (self.0.0, Distance::default()), |min, x| {
                let key_other = Key::from(*x);
                let compare = Key::from(self.0.0).distance(&key_other);
                if min.1 < Key::from(self.0.0).distance(&key_other) {
                    min
                } else {
                    (*x, compare)
                }
            }   );
        distance.0
    }
}