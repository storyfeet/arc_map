//! The ArcMap exists to enable multiple Mutex based Elements in a Map to be accessible
//! without the need for locking the whole HashMap, while you access one element.
//!
//! Instead the Map is hidden inside a thread, that will return accesible elements as requested.
//!
//! Rather than limit the access within the fn it seemed simple to return the Arc directly.
//!
//! Though because the Map may not contain the required element,
//! and becuase of thread send/recieve. The get method returns a "Result<Arc<Mutex<G>>,AMapErr>"
//!
//! This can be accessed as follows : *(Though normally for more complex objects)*
//!
//! ```
//! 
//! use arc_map::ArcMap;
//! let mut am = ArcMap::new();
//! let key = 3;
//!
//! am.insert(key,"hello".to_string());
//! {
//!     let p = am.get(3).unwrap(); //p is Arc<Mutex<String>>
//!     let mut s = p.lock().unwrap();
//!     s.push_str(" world");
//! }
//!
//! let p2 = am.get(3).unwrap();
//! let s2 = p2.lock().unwrap();
//! assert_eq!(*s2,"hello world".to_string());
//!
//! ```
//!
//! While this can be achieved using traditional mutex locks,
//! the interface here is much simpler to use.

use std::collections::HashMap;
use std::sync::mpsc::{channel,Sender};
use std::sync::{Arc,Mutex};
use std::thread;
use std::hash::Hash;

pub mod amap_error;
pub use amap_error::AMapErr;

pub trait MKey : 'static +Send + Eq + Hash{}

impl<K> MKey for K where K:'static+Send+Eq+Hash{}

enum Job<K:MKey,V:Send> {
    Add(K,V,Sender<bool>),
    Get(K,Sender<Option< Arc<  Mutex<V>  > >>),
    Remove(K),
}


#[derive(Clone)]
pub struct ArcMap<K:MKey,V:Send>{
    ch:Sender<Job<K,V>>,
}

fn hide_map<K:MKey,V:'static+Send>()->Sender<Job<K,V>>{
    let (tx,rx) = channel();
    thread::spawn(move ||{
        let mut mp:HashMap<K,Arc<Mutex<V>>> = HashMap::new();
        loop {
            match rx.recv(){
                Ok(Job::Add(k,v,cbak))=>{
                    mp.insert(k,Arc::new(Mutex::new(v)));
                    cbak.send(true).unwrap_or(());

                }
                Ok(Job::Get(ref k,ref cbak))=>{
                    match mp.get(k) {
                        Some(ref a)=>cbak.send(Some((*a).clone())).unwrap_or(()),
                        None=>cbak.send(None).unwrap_or(()),
                    }
                }
                Ok(Job::Remove(ref k))=>{
                    mp.remove(k);
                }
                //If all senders are dropped, kill the thread
                _=>return, 
            }
        }
    });
    tx 
}

impl<K:MKey,V:'static+Send> ArcMap<K,V>{
    pub fn new()->ArcMap<K,V>{
        ArcMap{
            ch:hide_map(),
        }
    }
    
    pub fn insert(&mut self, k:K,v:V)->Result<bool,AMapErr>{
        let (tbak,rbak) = channel();
        self.ch.send(Job::Add(k,v,tbak))?;
        Ok(rbak.recv()?)
    }

    pub fn get(&mut self, k:K)->Result<Arc<Mutex<V>>,AMapErr>{
        let (tbak,rbak) = channel();
        self.ch.send(Job::Get(k,tbak))?;
        match rbak.recv() {
            Ok(Some(a))=>Ok(a),
            Ok(None)=>Err(AMapErr::NotFound),
            Err(d)=>Err(d.into()),
        }
    }

    pub fn remove(&mut self, k:K)->Result<(),AMapErr>{
        self.ch.send(Job::Remove(k))?;
        Ok(())
    }

}



#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn add_to_ten(){
        let mut mp = ArcMap::new();

        for i in 0..10 {
            mp.insert(i,0).unwrap();
        }

        let mut handlers = Vec::new();
        for i in 0 .. 10{
            for _ in 0 .. 10 {
                let n = i;
                let m2 = mp.clone();
                let h = thread::spawn(move||{
                    let g = m2.clone().get(n).unwrap();
                    let mut incnum = g.lock().unwrap();
                    *incnum +=1;
                });
                handlers.push(h);
            }
        }

        for h in handlers {
            h.join().unwrap();
        }

        for i in 0..10 {
            let g = mp.get(i).unwrap();
            let num = g.lock().unwrap();
            assert_eq!(*num,10);
        }
        
    }
}




