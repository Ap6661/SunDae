use std::thread;

use rodio::{Decoder, OutputStream, Sink};
use slib::Item;
use crate::{singleton::Singleton, songmanager::SONGMANAGER, status::STATUS};

pub static PLAYBACK: Singleton<Playback> = Singleton::new(Playback::new);

pub struct Playback {
    _stream: OutputStream,
    sink: Sink,
    playback_running: bool,
}

impl Playback {
    pub fn new() -> Playback
    {
        // Make rodio stream
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        Playback {
            _stream: stream,
            sink,
            playback_running: false,
        }
    }

    pub fn queue_insert(&self, song: Item, index: usize ) {
        let index = index.clamp(0, STATUS.get().queue.len());
        STATUS.mutate( | s: &mut slib::Status | {
            s.queue.insert(index, song.clone());
        });
    }

    // pub fn queue_append(&mut self, song: Item) {
    //     self.queue_insert(song, self.status.queue.len());
    // }

    pub fn pause(&self) {
        self.sink.pause();
        STATUS.mutate( | s: &mut slib::Status | {
            s.playing = false;
        });
    }

    pub fn play(&self) {
        STATUS.mutate( | s: &mut slib::Status | {
            if !s.playing
            {
                s.playing = true;
                self.sink.play();
            }
        });

        if !self.playback_running {
            PLAYBACK.mutate( | p: &mut Playback | {
                p.playback_running = true;
            });
            playback();
        }
    }

    pub fn stop(&self) {
        self.sink.stop();
        self.pause();
        STATUS.mutate( | s: &mut slib::Status | {
            s.queue.clear();
            s.current_song = None;
        });
    }

    pub fn skip(&self) {
        self.play();
        self.sink.skip_one();
    }

    pub fn volume_adjust(&self, amount: f32) {
        self.volume_set(self.sink.volume() + amount);
    }

    pub fn volume_set(&self, amount: f32) {
        self.sink.set_volume(amount);
        STATUS.mutate( | s: &mut slib::Status | {
            s.volume = amount;
        });
    }

    pub fn restart(&self) {
        let song = STATUS.get().current_song.clone();
        match song {
            Some(s) => {
                self.queue_insert(s, 0);
                self.skip();
            },
            None => {},
        }
    }

    pub fn queue_remove(&self, index: u8) {
        STATUS.mutate( | s: &mut slib::Status | {
            s.queue.remove(index as usize);
        });
    }

}

fn playback() {
    // Run in background
    thread::spawn( || { 
        while STATUS.get().playing && STATUS.get().queue.len() != 0
        {
            STATUS.mutate( | s: &mut slib::Status | {
                // Set new current song
                let current_song = s.queue.pop_front().unwrap();
                s.current_song = Some(current_song.clone());
                let f = Decoder::new(SONGMANAGER.get().get_song(current_song)).unwrap();
                PLAYBACK.get().sink.append(f);
            });

            // NEVER WAIT WHILE YOU HAVE A LOCK
            PLAYBACK.get().sink.sleep_until_end();
        }
            STATUS.mutate( | s: &mut slib::Status | {
                // Wait till over
                s.current_song = None;
                s.playing = false;
                PLAYBACK.get().sink.stop();
            });

            PLAYBACK.mutate( | p: &mut Playback | {
                p.playback_running = false;
            });
    });
}
