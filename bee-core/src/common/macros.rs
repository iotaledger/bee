//! Common macros

#[cfg(test)]
macro_rules! sleep {
    ($duration:expr) => {
        std::thread::sleep(std::time::Duration::from_millis($duration));
    };
}

#[cfg(test)]
macro_rules! sleep_ns {
    ($duration:expr) => {
        std::thread::sleep(std::time::Duration::from_nanos($duration));
    };
}

macro_rules! unlock {
    ($mutex:expr) => {{
        $mutex.lock().expect("error taking the lock")
    }};
}

macro_rules! shared {
    ($inner:expr) => {{
        std::sync::Arc::new($inner)
    }};
}

macro_rules! shared_mut {
    ($inner:expr) => {{
        std::sync::Arc::new(std::sync::Mutex::new($inner))
    }};
}
