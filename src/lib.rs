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
pub struct GuardMap<K:MKey,V:Send>{
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
                _=>return,
            }
        }
    });
    tx 
}

impl<K:MKey,V:'static+Send> GuardMap<K,V>{
    pub fn new()->GuardMap<K,V>{
        GuardMap{
            ch:hide_map(),
        }
    }
    
    pub fn add_key(&mut self, k:K,v:V)->Result<bool,AMapErr>{
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
        let mut mp = GuardMap::new();

        for i in 0..10 {
            mp.add_key(i,0).unwrap();
        }

        for i in 0 .. 10{
            for _ in 0 .. 10 {
                let n = i;
                let m2 = mp.clone();
                thread::spawn(move||{
                    let g = m2.clone().get(n).unwrap();
                    let mut incnum = g.lock().unwrap();
                    *incnum +=1;
                });
            }
        }

        for i in 0..10 {
            let g = mp.get(i).unwrap();
            let num = g.lock().unwrap();
            assert_eq!(*num,10);
        }
        
    }
}




