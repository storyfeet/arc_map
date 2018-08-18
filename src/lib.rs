
use std::collections::HashMap;
use std::sync::mspc::{channel,Sender};
use std::Thread;

enum Job<K,V> {
    Add(K,V,Sender<bool>),
    Get(K,Sender<Arc<Mutex<V>>>),
    Remove(K),
    Kill,
}


pub struct GuardMap<K,V>{
    ch:Sender<Job<K,V>>,
}

fn hidemap<K,V>()->Sender<Job<G>>{
    let (tx,rx) = channel();
    thread::spawn(move ||{
        let mp:HashMap<usize,Arc<Mutex<G>>
        loop {
            
            

        }

    });
    

}

impl<G> GuardMap<G>{
    pub fn new()->GuardMap<G>{
        GuardMap{
            v:Hashmap::new(),
            ct:thread::spawn()
        }
    }
    
    add(&mut self, G)->usize{
        v:
    }
}



