#![allow(dead_code)] // not used on all platforms

#[cfg(not(target_os = "horizon"))]
use crate::mem;

#[cfg(target_os = "horizon")]
use crate::collections::BTreeMap;
#[cfg(target_os = "horizon")]
use crate::ptr;
#[cfg(target_os = "horizon")]
use crate::sync::atomic::{AtomicUsize, ATOMIC_USIZE_INIT, Ordering};

#[cfg(target_os = "horizon")]
pub type Key = usize;

#[cfg(not(target_os = "horizon"))]
pub type Key = libc::pthread_key_t;

#[cfg(target_os = "horizon")]
type Dtor = unsafe extern fn(*mut u8);

#[cfg(target_os = "horizon")]
static NEXT_KEY: AtomicUsize = ATOMIC_USIZE_INIT;

#[cfg(target_os = "horizon")]
static mut KEYS: *mut BTreeMap<Key, Option<Dtor>> = ptr::null_mut();

#[cfg(target_os = "horizon")]
#[thread_local]
static mut LOCALS: *mut BTreeMap<Key, *mut u8> = ptr::null_mut();

#[cfg(target_os = "horizon")]
unsafe fn keys() -> &'static mut BTreeMap<Key, Option<Dtor>> {
    if KEYS == ptr::null_mut() {
        KEYS = Box::into_raw(Box::new(BTreeMap::new()));
    }
    &mut *KEYS
}

#[cfg(target_os = "horizon")]
unsafe fn locals() -> &'static mut BTreeMap<Key, *mut u8> {
    if LOCALS == ptr::null_mut() {
        LOCALS = Box::into_raw(Box::new(BTreeMap::new()));
    }
    &mut *LOCALS
}

#[cfg(target_os = "horizon")]
#[inline]
pub unsafe fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    let key = NEXT_KEY.fetch_add(1, Ordering::SeqCst);
    keys().insert(key, dtor);
    key
}

#[cfg(not(target_os = "horizon"))]
#[inline]
pub unsafe fn create(dtor: Option<unsafe extern "C" fn(*mut u8)>) -> Key {
    let mut key = 0;
    assert_eq!(libc::pthread_key_create(&mut key, mem::transmute(dtor)), 0);
    key
}

#[cfg(target_os = "horizon")]
#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    locals().insert(key, value);
}

#[cfg(not(target_os = "horizon"))]
#[inline]
pub unsafe fn set(key: Key, value: *mut u8) {
    let r = libc::pthread_setspecific(key, value as *mut _);
    debug_assert_eq!(r, 0);
}

#[cfg(target_os = "horizon")]
#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    if let Some(&entry) = locals().get(&key) {
        entry
    } else {
        ptr::null_mut()
    }
}

#[cfg(not(target_os = "horizon"))]
#[inline]
pub unsafe fn get(key: Key) -> *mut u8 {
    libc::pthread_getspecific(key) as *mut u8
}

#[cfg(target_os = "horizon")]
#[inline]
pub unsafe fn destroy(key: Key) {
    keys().remove(&key);
}

#[cfg(not(target_os = "horizon"))]
#[inline]
pub unsafe fn destroy(key: Key) {
    let r = libc::pthread_key_delete(key);
    debug_assert_eq!(r, 0);
}

#[inline]
pub fn requires_synchronized_create() -> bool {
    false
}
