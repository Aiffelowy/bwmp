use rand::{seq::SliceRandom, thread_rng};
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::{collections::VecDeque, fmt, fs, io, path::PathBuf, time::Duration};

use crate::{sepuku, ui};

#[derive(Clone, Copy, Default)]
pub enum Repeat {
    #[default] None,
    Single,
    All,
}
impl fmt::Display for Repeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Repeat::*;
        match self {
            None => write!(f, "None"),
            Single => write!(f, "Single"),
            All => write!(f, "All"),
        }
    }
}

pub struct Player {
    _stream: OutputStream,
    output_stream_handle: OutputStreamHandle,
    music_list: Vec<PathBuf>,
    shuffled_list: Vec<u16>,
    pub queue: VecDeque<u16>,
    now_playing: Sink,
    now_playing_id: u16,
    repeat: Repeat,
    shuffle: bool,
    pub ui: ui::Ui,
    duration: Duration,
    time_playing: Duration,
}

impl Player {
    pub fn new(path: &str, repeat: Repeat, shuffle: bool, vol: f32) -> Self {
        let (stream, stream_handle) =
            OutputStream::try_default().expect("couldnt find default output device");
        let (sink, _) = Sink::new_idle();
        let mlist = match create_music_list(path) {
            Ok(m) => m,
            Err(e) => {sepuku(); panic!("{e}")}
        };
        let shuffled = shuffle_list(mlist.len() as u16);
        sink.set_volume(vol);
        Self {
            _stream: stream,                     //must not be freed
            output_stream_handle: stream_handle, //also must not be freed
            ui: ui::Ui::new(mlist.clone()),
            music_list: mlist,
            shuffled_list: shuffled,
            queue: VecDeque::new(),
            now_playing: sink,
            now_playing_id: 0,
            repeat: repeat,
            shuffle: shuffle,
            duration: Duration::new(0, 0),
            time_playing: Duration::new(0, 0),
        }
    }
    //toggles shuffle
    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        self.ui.update_shuffle(self.shuffle);
    }
    //cycles throu repeat options, alternatively you can set a specific one
    pub fn toggle_repeat(&mut self, o: Option<Repeat>) {
        if o.is_some() {
            self.repeat = o.unwrap();
            self.ui.update_repeat(self.repeat);
            return;
        }
        use Repeat::*;
        self.repeat = match self.repeat {
            Repeat::None => Single,
            Single => All,
            All => Repeat::None,
        };
        self.ui.update_repeat(self.repeat);
    }
    //returns current volume
    pub fn volume(&self) -> f32 {
        self.now_playing.volume()
    }
    //changes volume to a specific float value
    pub fn change_volume(&mut self, volume: f32) {
        if volume >= -0.01 && volume <= 2.01 {
            self.now_playing.set_volume(volume);
            self.ui.update_volume(volume);
        }
    }
    //toggled pause
    pub fn toggle_pause(&mut self) {
        if self.now_playing.is_paused() {
            self.now_playing.play();
        } else {
            self.now_playing.pause();
        }
        self.ui.update_status(self.now_playing.is_paused());
    }
    //check is the player is playing or paused
    pub fn is_playing(&self) -> bool {
        !self.now_playing.is_paused()
    }
    //check if sink has finished playing
    pub fn is_finished(&self) -> bool {
        self.now_playing.empty()
    }
    pub fn put_selected_in_queue(&mut self) {
        self.queue.push_back(self.ui.get_selected());
        self.ui.update_queue(&self.queue);
    }
    //play a file with a specific id within a list
    pub fn play(&mut self, id: u16) {
        let file = fs::File::open(self.music_list[id as usize].clone()).unwrap();
        let name = self.music_list[id as usize]
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        self.duration = mp3_duration::from_file(&file).unwrap();
        self.time_playing = Duration::new(0, 0);
        self.ui.change_name(name);
        self.ui.highlight_playing(id);
        self.now_playing = self.output_stream_handle.play_once(file).unwrap();
        self.now_playing.set_volume(self.volume());
    }
    //plays previous file in list, does nothing when the first file is playing
    pub fn play_previous(&mut self) {
        if self.now_playing_id == 0 {
            return;
        }
        self.time_playing = Duration::new(0, 0);
        self.now_playing_id -= 1;
        let mut id = self.now_playing_id;
        if self.shuffle {
            id = self.shuffled_list[id as usize]
        }

        self.play(id);
    }
    //plays next file in list, repeat value alters the behavior
    pub fn play_next(&mut self, skip: bool) {
        if !self.queue.is_empty() {
            self.now_playing_id = (self.queue.pop_front().unwrap() - 1).into();
            self.ui.update_queue(&self.queue);
            self.play(self.now_playing_id);
            return;
        }
        match self.repeat {
            Repeat::None => {
                if self.now_playing_id + 1 >= self.music_list.len() as u16 {
                    return;
                }
                self.now_playing_id += 1
            }
            Repeat::Single => {
                if skip {
                    self.now_playing_id += 1
                }
            }
            Repeat::All => {
                if self.now_playing_id + 1 >= self.music_list.len() as u16 {
                    self.now_playing_id = 0;
                } else {
                    self.now_playing_id += 1;
                }
            }
        }

        self.time_playing = Duration::new(0, 0);
        let id = if self.shuffle {
            self.shuffled_list[self.now_playing_id as usize]
        } else {
            self.now_playing_id
        };
        self.play(id);
    }
    pub fn play_selected(&mut self) {
        self.now_playing_id = self.ui.get_selected() - 1;
        self.play((self.ui.get_selected() - 1).into());
    }
    //updates the time bar
    pub fn update_bar(&mut self, t: Duration) {
        self.ui.update_bar(self.duration, self.time_playing);
        self.time_playing = self.time_playing.saturating_add(t);
    }
    pub fn search_for(&mut self, search_str: &str) {
        self.ui.display_searching(search_str);
        if search_str.len() > 2 {
            self.ui.highlight_searched(search_str);
        }
    }
}

//creates a music list from a given path
fn create_music_list(path: &str) -> Result<Vec<PathBuf>, io::Error> {
    let mut paths: Vec<PathBuf> = fs::read_dir(path)?
        .map(|res| res.map(|p| p.path()))
        .collect::<Result<Vec<_>, io::Error>>()?;
    paths.retain(|m| match m.extension() {
        Some(p) => p.to_os_string().eq("mp3"),
        None => false,
    });
    if paths.len() == 0 {
        return Err(io::Error::new(io::ErrorKind::NotFound, "no music files found!"));
    }
    Ok(paths)
}

//returns a vec of random numbers, used to play random music
fn shuffle_list(len: u16) -> Vec<u16> {
    let mut shuffled: Vec<u16> = (0..len).collect();
    shuffled.shuffle(&mut thread_rng());
    shuffled
}
