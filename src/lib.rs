//! The ArcMap exists to enable multiple Mutex based Elements in a Map to be accessible
//!
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
//! use arc_map::ArcMap;
//! let mut am = ArcMap::new();
//!
//! //update by grabbing mutex  
//! am.insert(3,"hello".to_string());
//! {
//!     let p = am.get(3).unwrap(); //p is Arc<Mutex<String>>
//!     let mut s = p.lock().unwrap();
//!     s.push_str(" world");
//! }
//!
//! //read by grabbing mutex 
//! { 
//!     let p2 = am.get(3).unwrap();
//!     let s2 = p2.lock().unwrap();
//!     assert_eq!(*s2,"hello world".to_string());
//! }
//! 
//! 
//! am.insert(4,"goodbye".to_string());
//!
//! //update in place (No need for scoping)
//! am.on_do(4,|mut s| (&mut s).push_str(" cruel world")).unwrap();
//!
//! //get info out
//! let ls = am.on_do(4,|s| s.clone()).unwrap();
//! 
//! assert_eq!(&ls,"goodbye cruel world");
//!
//! ```
//!
//! While this can be achieved using traditional mutex locks,
//! the interface here is much simpler to use.

use std::collections::HashMap;
use std::sync::mpsc::{channel,Sender,sync_channel,SyncSender};
use std::sync::{Arc,Mutex};
use std::thread;
use std::hash::Hash;
use std::fmt::Debug;

pub mod amap_error;
pub use amap_error::AMapErr;

pub trait MKey : 'static +Send + Eq + Hash{}
impl<K> MKey for K where K:'static+Send+Eq+Hash{}

pub trait MVal: 'static +Send {}
impl <V> MVal for V where V:'static+ Send + Debug{}



/// internal type for sending across job channel
enum Job<K:MKey,V:MVal> {
    Add(K,V,Sender<bool>),
    Get(K,Sender<Option< Arc<  Mutex<V>  > >>),
    Remove(K),
}


#[derive(Clone)]
pub struct ArcMap<K:MKey,V:MVal>{
    ch:SyncSender<Job<K,V>>,
}

fn hide_map<K:MKey,V:MVal>(bsize:usize)->SyncSender<Job<K,V>>{
    let (tx,rx) = sync_channel(bsize);
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

impl<K:MKey,V:MVal> ArcMap<K,V>{
    /// Create a new ArcMap This will spawn a guard process
    /// That process will die when the last Clone of this map is dropped
    pub fn new()->ArcMap<K,V>{
        ArcMap{
            ch:hide_map(20),//reasonable default
        }
    }

    pub fn new_sized(bsize:usize)->ArcMap<K,V>{
        ArcMap{
            ch:hide_map(bsize),
        }
    }

    
    /// Add a new item to the map. 
    pub fn insert(&self, k:K,v:V)->Result<bool,AMapErr>{
        let (tbak,rbak) = channel();
        self.ch.send(Job::Add(k,v,tbak))?;
        Ok(rbak.recv()?)
    }

    /// The basic way of getting an item out of the list for editing
    /// returns type of (wrapped) Arc means you can keep this after closing and still be safe
    /// In general prefer on_do
    pub fn get(&self, k:K)->Result<Arc<Mutex<V>>,AMapErr>{
        let (tbak,rbak) = channel();
        self.ch.send(Job::Get(k,tbak))?;
        match rbak.recv() {
            Ok(Some(a))=>Ok(a),
            Ok(None)=>Err(AMapErr::NotFound),
            Err(d)=>Err(d.into()),
        }
    }

    /// This function only removes the object from the list.
    /// If you have an Arc Copy that will still be valid
    pub fn remove(&self, k:K)->Result<(),AMapErr>{
        self.ch.send(Job::Remove(k))?;
        Ok(())
    }

    /// Run f on the item at the index "on",
    /// Returns the result of f wrapped in a Result
    /// Allows for reading data out of the object
    /// Errors if index not found, or channel/locking errors
    pub fn on_do<RT,F>(&self, on:K,f:F)->Result<RT,AMapErr>
        where F:FnOnce(&mut V)->RT
    {
        let p = self.get(on)?; 
        let mut v = p.lock()?;
        Ok(f(&mut v))
    }
}



#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn add_to_ten(){
        let mp = ArcMap::new();

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

        for i in 0 ..10{
            for _ in 0 ..10{
                let n = i;
                let m2 = mp.clone();
                let h = thread::spawn(move||{
                    m2.clone().on_do(n,|v|*v += 1).unwrap();
                });
                handlers.push(h);
            }
        }

        for h in handlers {
            h.join().unwrap();
        }

        for i in 0..10 {
            mp.on_do(i,|num| assert_eq!(*num,20)).unwrap();
            //same but other method
            let g = mp.get(i).unwrap();
            let num = g.lock().unwrap();
            assert_eq!(*num,20);
        }
    }
}




