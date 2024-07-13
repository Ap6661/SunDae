mod playback;
mod songmanager;
mod config;
mod singleton;
mod status;

use std::u8;

use config::CONFIG;
use playback::PLAYBACK;
use reqwest::blocking::Client;
use slib::{Daemon, Item, SongInfo, Status};
use status::STATUS;
use subsonic_types::{common::{Format, Version}, request::{Request, SubsonicRequest}, response::ResponseBody};



struct Server {
    reqwest_client: Client,
    /// For Caching. To update please call Scan
    fetched_songs: Vec<Item>,
    fetched_albums: Vec<Item>,
    fetched_artists: Vec<Item>,
}

impl Server {
    fn new() -> Option<Self>
    {
        // let songmanager = SongManager::new(&config, &authentication);

        let server = Self {
                reqwest_client: reqwest::blocking::Client::new(),
                fetched_songs: vec!(),
                fetched_albums: vec!(),
                fetched_artists: vec!(),
        };

        
        // Ping to to verify credentials and server
        let response = server.send(subsonic_types::request::system::Ping {});
        if response.unwrap().body == subsonic_types::response::ResponseBody::Empty
        {
            Some(server)
        }
        else
        {
            None
        }
    }

    fn send<S: SubsonicRequest>(&self, body: S) -> Result<subsonic_types::response::Response, reqwest::Error> {
        let request = Request {
            username: CONFIG.get().config.username.clone(),
            authentication: CONFIG.get().auth.clone(),
            version: Version::LATEST,
            client: "SunDae".into(),
            format: Some(Format::Json.to_string()),
            body,
        };
        let url = CONFIG.get().config.server.clone() + S::PATH + "?" + &request.to_query();
        let request = self.reqwest_client.get(url);
        let text = request.send().expect("Failed to send").text();
        Ok(subsonic_types::response::Response::from_json(text.unwrap().as_str()).expect("Failed to parse response"))
    }
}

impl Server {
        fn fetch_album_rec(&self, offset: u32) -> Vec<slib::Item> {
            let size = 500;
            let response = &self.send(subsonic_types::request::lists::GetAlbumList2 { 
                list_type: subsonic_types::request::lists::ListType::AlphabeticalByName,
                size: Some(size),
                offset: Some(offset),
                genre: None,
                to_year: None,
                from_year: None,
                music_folder_id: None,
            }).unwrap();

            match &response.body {
                ResponseBody::AlbumList2(l) => { 
                    let mut output: Vec<slib::Item> = l.album.iter().map( |i| {
                        slib::Item {
                            id: i.id.to_owned(),
                            name: i.name.to_owned(),
                            image_path: "None".into(),
                        }
                    }).collect();
                    if l.album.len() as u32 >= size
                    {
                        output.append(self.fetch_album_rec(offset + size).as_mut());
                        return output;
                    }
                    else
                    {
                        return output;
                    }
                }
                _ => {
                    return Vec::new();
                }
            }
        }
}

impl Daemon for Server {
        fn shutdown(&self)                                                   -> bool {
            true
        }

        fn fetch_artists(&mut self)                                         -> Vec<slib::Item> {
            if self.fetched_artists.len() == 0 
            {
                let _ = &self.scan();
            }
            self.fetched_artists.clone()
        }

        fn fetch_albums(&mut self)  -> Vec<slib::Item> {
            if self.fetched_albums.len() == 0 
            {
                let _ = &self.scan();
            }
            self.fetched_albums.clone()
        }


        fn fetch_playlists(&mut self)                                       -> Vec<slib::Item> {
            todo!()
        }

        fn fetch_songs(&mut self)                                           -> Vec<slib::Item> {
            dbg!("Fetching");
            if self.fetched_songs.len() == 0 
            {
                dbg!("Scanning");
                let _ = &self.scan();
            }
            self.fetched_songs.clone()
        }

        fn scan(&mut self)                                                       -> bool {

            self.fetched_albums = self.fetch_album_rec(0);
            
            self.fetched_songs = self.fetch_albums().iter().map ( | a | 
                match self.send( subsonic_types::request::browsing::GetAlbum {
                    id: a.id.to_owned(),
                }).unwrap().body
                {
                    ResponseBody::Album(l) => {
                        l.song.iter().map( | s | {
                            Item {
                                id: s.id.to_owned(),
                                name: s.title.to_owned(),
                                image_path: "None".into(),
                            }}).collect()
                    },
                    _ => { Vec::new() }
                }
            ).into_iter().flatten().collect();

            self.fetched_artists = match 
                self.send( subsonic_types::request::browsing::GetArtists{
                    music_folder_id: None
                }).unwrap().body
            {
                ResponseBody::Artists(l) => {
                    l.index.iter().map( | a | {
                        a.artist.iter().map(| b | Item {
                            name: b.name.to_owned(),
                            id: b.id.to_owned(),
                            image_path: "None".into()
                        }).into_iter()
                    }).into_iter().flatten().collect()
                },
                _ => { Vec::new() }
            };

            self.send( subsonic_types::request::scan::StartScan{}).unwrap();
            true
        }

        fn status(&self)                                                     -> &Status {
            STATUS.get()
        }

        fn restart(&self)                                                    -> bool {
            PLAYBACK.get().restart();
            true
        }

        fn play(&mut self)                                                   -> bool {
            PLAYBACK.get().play();
            true
        }

        fn stop(&mut self)                                                   -> bool {
            PLAYBACK.get().stop();
            true
        }

        fn pause(&mut self)                                                  -> bool {
            PLAYBACK.get().pause();
            true
        }

        fn skip(&mut self)                                                   -> bool {
            PLAYBACK.get().skip();
            true
        }

        fn queue_add(&mut self, id: slib::Item, position: u8)                -> bool {
            PLAYBACK.get().queue_insert(id, position as usize);
            return true
        }

        fn queue_remove(&mut self, index: u8)                           -> bool {
            PLAYBACK.get().queue_remove(index);
            true
        }

        fn volume_adjust(&mut self, amount: f32)                             -> bool {
            PLAYBACK.get().volume_adjust(amount);
            true
        }

        fn volume_set(&mut self, amount: f32)                                -> bool {
            dbg!("Volume is going to be set");
            PLAYBACK.get().volume_set(amount);
            dbg!("Volume is set");
            true
        }

        fn search(&self, query: String)                                      -> Vec<slib::Item> {
            todo!()
        }

        fn download(&self, id: slib::Item)                                   -> bool {
            todo!()
        }

        fn delete(&self, id: slib::Item)                                     -> bool {
            todo!()
        }

        fn star(&self, id: slib::Item)                                       -> bool {
            todo!()
        }

        fn playlist_download(&self, id: slib::Item)                          -> bool {
            todo!()
        }

        fn playlist_upload(&self, id: slib::Item)                            -> bool {
            todo!()
        }

        fn playlist_new(&self, name: String)                                 -> bool {
            todo!()
        }       

        fn playlist_add_to(&self, playlist: slib::Item, id: slib::Item)      -> bool {
            todo!()
        }

        fn playlist_remove_from(&self, playlist: slib::Item, id: slib::Item) -> bool {
            todo!()
        }

        fn playlist_delete(&self, id: slib::Item)                            -> bool {
            todo!()
        }

        fn song_info(&self, id: slib::Item)                                  -> Option<slib::SongInfo> {
           match self.send( subsonic_types::request::browsing::GetSong { 
               id: id.id 
           } ).unwrap().body
           {
               ResponseBody::Song(s) => {
                   Some(SongInfo {
                       length: s.duration.unwrap().to_duration(),
                       album: Item {
                           id: s.album_id.unwrap(),
                           name: s.album.unwrap(),
                           image_path: "None".into(),
                       },
                       artist: s.artist.unwrap()
                   })
               },
               _ => {
                   None
               }
           }
        }

        fn album_info(&self, id: slib::Item)                                 -> Option<slib::AlbumInfo> {
           match self.send( subsonic_types::request::browsing::GetAlbum { 
               id: id.id 
           } ).unwrap().body
           {
               ResponseBody::Album(a) => {
                   Some(slib::AlbumInfo {
                       artist: a.album.artist.unwrap().to_owned(),
                       songs: a.song.iter().map( | s | {
                               Item {
                                   id: s.id.to_owned(), name: s.title.to_owned(),
                                   image_path: "None".into(),
                               }}).collect()
                   })
               },
               _ => {
                   None
               }
           }
        }

}



fn main() {
    let mut server = Server::new().unwrap();
    server.start();
}

#[cfg(test)]
mod tests
{
    use super::*;
    use std::{thread, time::Duration};

    #[test]
    fn fetch() 
    {
        thread::spawn( move || {
            let mut server = Server::new().unwrap();
            server.start();
        });

        thread::sleep(Duration::from_secs(1));
        let client = slib::Client::new().unwrap();
        // dbg!(client.fetch_songs());

        // let song = client.fetch_songs().get(0).unwrap().clone();
        let song = Item { 
            name: "Song".into(),
            id: "b4f27bed147b7c5c34dfe2d9f706a284".into(),
            image_path: "None".into()
        };

        client.queue_add(song, 0);
        client.play();
        thread::sleep(Duration::from_secs(10));
        client.volume_set(0.2);
        thread::sleep(Duration::from_secs(1));
        client.volume_adjust(0.2);
        thread::sleep(Duration::from_secs(1));
        client.volume_adjust(0.2);
        thread::sleep(Duration::from_secs(1));
        client.volume_adjust(0.2);

        thread::sleep(Duration::from_secs(60));

        dbg!("this is on purpose sorry future me");
        unreachable!();
    }
}
