ArcMap
=======

Arc Map is designed to allow a HashMap of Arc\<Mutex\<T>> to be stored in such a way that elements can be accessed without the need to lock the whole Map while you have access to them.

**Contributions Welcome**

### changes in v0.1.4 - coming
changed on_do to take a &self, instead of &mut self 

### changes in v0.1.3

changed &mut self to &self, in three main methods.  making it much easier to use as part of another object.  

### changes in v0.1.2

Switched main channel to SyncSender, for sync in parents
