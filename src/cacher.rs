use std::{
    collections::BTreeMap,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use reqwest::blocking as http;
use tokio::runtime::Builder as TokioBuilder;
use tokio::runtime::Runtime as TokioRuntime;

#[allow(clippy::module_name_repetitions)]
pub type ByteCacher = Cacher<Vec<u8>>;

pub struct Cacher<T> {
    inner: BTreeMap<String, InternalCacheState<T>>,
    thread_pool: TokioRuntime,
}

impl<T> Cacher<T> {
    pub fn new() -> Self {
        Self {
            inner: BTreeMap::new(),
            thread_pool: TokioBuilder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap(),
        }
    }

    /// pure fn to tell what state the cache is in
    fn get_state(&self, key: &str) -> InternalCacheState<()> {
        match self.inner.get(key) {
            Some(InternalCacheState::Stored(_)) => InternalCacheState::Stored(()),
            Some(InternalCacheState::Calling) => InternalCacheState::Calling,
            Some(InternalCacheState::Empty) | None => InternalCacheState::Empty,
        }
    }

    /// pure fn to get a reference to the cache's internal state
    fn get(&self, key: &str) -> &InternalCacheState<T> {
        self.inner
            .get(key)
            .map_or_else(|| &InternalCacheState::Empty, |internal| internal)
    }
}

#[derive(Clone, Copy, Debug)]
enum InternalCacheState<T> {
    /// there's a stored value
    Stored(T),
    /// there's currently a thread trying to grab the data
    Calling,
    /// there's currently nothing happening
    Empty,
}

impl<T> InternalCacheState<T> {
    /// pure fn to convert to an option for if the state contains data
    #[allow(clippy::missing_const_for_fn)]
    fn try_stored(self) -> Option<T> {
        match self {
            Self::Stored(t) => Some(t),
            _ => None,
        }
    }
}

pub fn get_from_cache(cache: Arc<Mutex<ByteCacher>>, key: &str, verbose: bool) -> Option<Vec<u8>> {
    let state = cache.lock().unwrap().get_state(key);
    match state {
        // already cached a value => just return it
        InternalCacheState::Stored(_) => cache.lock().unwrap().get(key).clone().try_stored(),
        // cache is empty => return nothing but try to fill it for next time
        InternalCacheState::Empty => {
            // make sure we can move the key into the closure
            let key = String::from(key);
            // tell the cache that we're processing the request
            cache
                .lock()
                .unwrap()
                .inner
                .insert(key.clone(), InternalCacheState::Calling);
            cache.clone().lock().unwrap().thread_pool.spawn(async move {
                // get the value from the interwebs. If it works, this is the Stored value. If there's an error, make it empty
                if verbose {
                    print!("Closure Starting: get bytes from {key}\r\n");
                }
                let value = get_from_cache_blocking(&cache, &key)
                    .map_or(InternalCacheState::Empty, |vec| {
                        InternalCacheState::Stored(vec)
                    });
                if verbose {
                    print!("Got Value: {value:?}\r\n");
                }
                cache.lock().unwrap().inner.insert(key, value);
            });
            None
        }
        // we're in the middle of filling it => trust the process
        InternalCacheState::Calling => None,
    }
}

pub fn get_from_cache_blocking(
    cache: &Arc<Mutex<ByteCacher>>,
    key: &str,
) -> Result<Vec<u8>, String> {
    let state = cache.lock().unwrap().get_state(key);
    if let InternalCacheState::Stored(_) = state {
        cache
            .lock()
            .unwrap()
            .get(key)
            .clone()
            .try_stored()
            .ok_or_else(|| String::from("Internal Cache Error"))
    } else {
        let res = http::get(key)
            .or_else(|_| http::get(format!("https://{key}")))
            .or_else(|_| http::get(format!("https://www.{key}")))
            .map_err(|err| format!("Network Error: {err}"))?
            .bytes()
            .map_err(|err| format!("Decoding Error: {err}"))?
            .to_vec();
        cache
            .lock()
            .unwrap()
            .inner
            .insert(String::from(key), InternalCacheState::Stored(res.clone()));
        Ok(res)
    }
}
