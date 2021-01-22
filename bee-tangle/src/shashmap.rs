use std::{
    //collections::{hash_map::RandomState},
    hash::{Hash, Hasher, BuildHasher},
    time::Duration,
    thread,
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};
use hashbrown::{HashMap as InnerHashMap, hash_map::Entry};
use fxhash::FxBuildHasher as RandomState;
use spin::{Once, /*RwLock*/};
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

type InnerMap<K, V> = InnerHashMap<Key<K>, V, NoHasherBuilder>;

fn wait_for<'a, R>(mut f: impl FnMut() -> Option<R>) -> R {
    loop {
        if let Some(r) = f() {
            return r;
        }
        thread::yield_now();
    }
}

const SHARDS: usize = 64;

pub struct HashMap<K, V, S = RandomState> {
    shards: [Once<RwLock<InnerMap<K, V>>>; SHARDS],
    build_hasher: S,
    len: AtomicUsize,
}

impl<K, V> HashMap<K, V, RandomState> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<K, V, S> HashMap<K, V, S> {
    pub fn with_capacity_and_hasher(cap: usize, build_hasher: S) -> Self {
        Self {
            shards: [Once::INIT; SHARDS],
            build_hasher,
            len: AtomicUsize::new(0),
        }
    }
}

impl<K: Hash + Eq, V, S: BuildHasher> HashMap<K, V, S> {
    fn shard_for(&self, hash: u64) -> &RwLock<InnerMap<K, V>> {
        let index = (hash % SHARDS as u64) as usize;
        self.shards[index].call_once(|| RwLock::new(InnerMap::default()))
    }

    async fn with_shard<R>(&self, hash: u64, f: impl FnOnce(&InnerMap<K, V>) -> R) -> R {
        f(&*self.shard_for(hash).read().await)
    }

    async fn with_shard_mut<R>(&self, hash: u64, f: impl FnOnce(&mut InnerMap<K, V>) -> R) -> R {
        f(&mut *self.shard_for(hash).write().await)
    }

    pub async fn insert(&self, key: K, value: V) -> Option<V> {
        let key = Key::new(key, &self.build_hasher);
        let r = self.with_shard_mut(key.hash, |shard| shard.insert(key, value)).await;
        if r.is_none() {
            self.len.fetch_add(1, Ordering::SeqCst);
        }
        r
    }

    pub async fn remove(&self, key: &K) -> Option<V> where K: Clone {
        let key = Key::new(key.clone(), &self.build_hasher);
        let r = self.with_shard_mut(key.hash, |shard| shard.remove(&key)).await;
        if r.is_some() {
            self.len.fetch_sub(1, Ordering::SeqCst);
        }
        r
    }

    pub async fn contains_key(&self, key: &K) -> bool where K: Clone, V: Clone {
        let key = Key::new(key.clone(), &self.build_hasher);
        self.shard_for(key.hash).read().await.contains_key(&key)
    }

    pub async fn len(&self) -> usize where K: Clone, V: Clone {
        self.len.load(Ordering::Relaxed)
    }

    pub async fn get(&self, key: &K) -> Option<impl Deref<Target=V> + '_> where K: Clone, V: Clone {
        let key = Key::new(key.clone(), &self.build_hasher);
        RwLockReadGuard::try_map(self.shard_for(key.hash).read().await, |m| m.get(&key)).ok()
    }

    pub async fn do_for_mut<R>(&self, key: &K, f: impl FnOnce(&mut V) -> R) -> Option<R> where K: Clone, V: Clone {
        let key = Key::new(key.clone(), &self.build_hasher);
        self.with_shard_mut(key.hash, |shard| shard.get_mut(&key).map(f)).await
    }

    pub async fn get_cloned(&self, key: &K) -> Option<V> where K: Clone, V: Clone {
        let key = Key::new(key.clone(), &self.build_hasher);
        //self.with_shard(key.hash, |shard| shard.get(&key).cloned())
        self.shard_for(key.hash).read().await.get(&key).cloned()
    }

    // pub async fn entry(&mut self, key: K) -> impl Deref<Target=Entry<'_, Key<K>, V, NoHasherBuilder>> + '_ where K: Clone, V: Clone {
    //     let key = Key::new(key.clone(), &self.build_hasher);
    //     //self.with_shard(key.hash, |shard| shard.get(&key).cloned())

    //     RwLockWriteGuard::map(self.shard_for(key.hash).write().await, |m| &mut m.entry(key))
    // }
}

impl<K, V, S: BuildHasher + Default> Default for HashMap<K, V, S> {
    fn default() -> Self {
        Self::with_capacity_and_hasher(0, S::default())
    }
}

#[derive(PartialEq, Eq)] // TODO: Implement just on `inner`
struct Key<K> {
    pub hash: u64,
    pub inner: K,
}

impl<K: Hash> Key<K> {
    fn new<B: BuildHasher>(inner: K, bh: &B) -> Self {
        let mut hasher = bh.build_hasher();
        inner.hash(&mut hasher);
        Self {
            hash: hasher.finish(),
            inner,
        }
    }
}

impl<K> Hash for Key<K> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

#[derive(Default)]
struct NoHasherBuilder;

impl BuildHasher for NoHasherBuilder {
    type Hasher = NoHasher;

    fn build_hasher(&self) -> Self::Hasher {
        NoHasher(0)
    }
}

struct NoHasher(u64);

impl Hasher for NoHasher {
    fn finish(&self) -> u64 { self.0 }
    fn write(&mut self, bytes: &[u8]) { todo!() }
    fn write_u64(&mut self, x: u64) { self.0 = x; }
}
