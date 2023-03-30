use scatter_gather_models::peer_id::PeerId;
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