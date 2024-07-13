use std::collections::VecDeque;
use crate::singleton::Singleton;

pub static STATUS: Singleton<slib::Status> = Singleton::new( || { 
    slib::Status {
        playing: false,
        current_song: None,
        queue: VecDeque::new(),
        volume: 1.0,
    }
});
