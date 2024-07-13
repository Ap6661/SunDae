use std::{fs::File, io::{BufReader, Write}, path::PathBuf};

use reqwest::blocking::Client;
use slib::Item;
use subsonic_types::{common::{Format, Version}, request::{Request, SubsonicRequest}};
use tempfile::TempDir;
use crate::{config::CONFIG, singleton::Singleton};

pub static SONGMANAGER: Singleton<SongManager> = Singleton::new(SongManager::new);

pub struct SongManager {
    temp_directory: TempDir,
    reqwest_client: Client,
}

impl SongManager {
    
    pub fn new() -> Self {
        Self {
            temp_directory: TempDir::new().expect("Failed to make directory"),
            reqwest_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn get_song(&self, song: Item) -> BufReader<File> {
        // Check if song is already downloaded ## TODO
        let mut f = None;
        if self.temp_directory.path().join(song.id.clone()).exists()
        {
            f = Some(self.temp_directory.path().join(song.id.clone()));
        }
        // If not download it as temp.
        if f.is_none() 
        {
            f = Some(self.temp_download(&song).unwrap());
        }

        // Would fail if song doesn't exist 
        BufReader::new( File::open(unsafe { 
            f.unwrap_unchecked()
        }).unwrap())
    }

    fn temp_download(&self, song: &Item) -> std::io::Result<PathBuf>
    {
        let request = Request {
            username: CONFIG.get().config.username.to_string(),
            authentication: CONFIG.get().auth.to_owned(),
            version: Version::LATEST,
            client: "SunDae".into(),
            format: Some(Format::Json.to_string()),
            body: subsonic_types::request::retrieval::Download {
                id: song.id.clone()
            }
        };
        let url = CONFIG.get().config.server.to_owned() + 
            subsonic_types::request::retrieval::Download::PATH + 
            "?" + &request.to_query();

        let response = self.reqwest_client.get(url).send().unwrap();
        let path = self.temp_directory.path().join(song.id.clone());
        let mut file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&path)?;

        let content = response.bytes().unwrap();
        file.write_all(&content)?;
        Ok(path)
    }

}
