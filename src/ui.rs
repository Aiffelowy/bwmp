use std::{
    collections::VecDeque,
    io::{stdout, Write},
    path::PathBuf,
    time::Duration,
};

use termion::color::{self, Bg, Fg};
use termion::cursor::{DetectCursorPos, Goto};

use crate::{colors::Colors, misc::clear_term, misc::cut_string, player::Repeat};

const RESET_FG: Fg<color::Reset> = color::Fg(color::Reset);
const RESET_BG: Bg<color::Reset> = color::Bg(color::Reset);

pub struct Ui {
    stdout: std::io::Stdout,
    colors: Colors,
    lines: u16,
    offset: u16,
    cursor_pos: u16,
    pub now_playing: u16,
    music_list_names: Vec<String>,
    term_size: [u16; 2],
    name_coords: [u16; 2],
    status_coords: [u16; 2],
    repeat_coords: [u16; 2],
    shuffle_coords: [u16; 2],
    volume_coords: [u16; 2],
    queue_coords: [u16; 2],
    bar_coords: [u16; 2],
}

impl Ui {
    //init the ui; I'm sooo fucking sorry
    pub fn new(music_list: Vec<PathBuf>) -> Self {
        let mut stdout = stdout();
        let colors = Colors::new_from_config().unwrap_or(Colors::new_default());
        let (term_x, term_y) = termion::terminal_size().unwrap();
        let l: u16 = if music_list.len() > (term_y - 7).into() {
            term_y - 7
        } else {
            music_list.len().try_into().unwrap() //shouldnt panic (well it will if your terminal
                                                 //has more than 2^16 lines, but how would you even do that)
        };
        let mls: Vec<String> = music_list
            .iter()
            .map(|p| cut_string(p.file_name().unwrap().to_string_lossy().to_string(), (term_x - 8).into()))
            .collect();
        clear_term();
        print!("{}", Goto(1, 1));
        draw_horizontal_lines(term_x, "¯");
        init_music_list(&mls, l);
        print!("\n\r");
        print!("{}", Goto(1, term_y - 5));
        draw_horizontal_lines(term_x, "-");
        print!("\n\r {}Now playing: ", Fg(colors.name));
        let (name_x, name_y) = stdout.cursor_pos().unwrap();
        print!("\r\n ");
        let (status_x, status_y) = stdout.cursor_pos().unwrap();
        print!("{}Playing  ", Fg(colors.status));
        print!("{}Repeat: ", Fg(colors.repeat));
        let (repeat_x, repeat_y) = stdout.cursor_pos().unwrap();
        print!("None    ");
        print!("{}Shuffle: ", Fg(colors.shuffle));
        let (shuffle_x, shuffle_y) = stdout.cursor_pos().unwrap();
        print!("false  ");
        print!("{}Volume: ", Fg(colors.volume));
        let (vol_x, vol_y) = stdout.cursor_pos().unwrap();
        print!("100  ");
        let (queue_x, queue_y) = stdout.cursor_pos().unwrap();
        print!("{}Queue: ", Fg(colors.queue));
        print!("empty");
        print!("\r\n\n ");
        let (bar_x, bar_y) = stdout.cursor_pos().unwrap();
        print!("\r\n{}", RESET_FG);
        draw_horizontal_lines(term_x, "_");
        draw_vertical_lines([term_x, term_y]);
        stdout.flush().unwrap();

        Self {
            stdout: stdout,
            colors: colors,
            music_list_names: mls,
            lines: l,
            offset: 0,
            cursor_pos: 0,
            now_playing: 0,
            term_size: [term_x, term_y],
            name_coords: [name_x, name_y],
            status_coords: [status_x, status_y],
            repeat_coords: [repeat_x, repeat_y],
            shuffle_coords: [shuffle_x, shuffle_y],
            volume_coords: [vol_x, vol_y],
            queue_coords: [queue_x, queue_y],
            bar_coords: [bar_x, bar_y],
        }
    }
    //changes the name of the track
    pub fn change_name(&mut self, name: &str) {
        self.clear_area(
            self.name_coords[0],
            self.name_coords[1],
            self.term_size[0] - 1,
            self.name_coords[1],
        );
        print!(
            "{}{}{}",
            Goto(self.name_coords[0], self.name_coords[1]),
            Fg(self.colors.name),
            cut_string(name.into(), (self.term_size[0]-17).into())
        );
        self.stdout.flush().unwrap();
    }
    //changes the status of the track
    pub fn update_status(&mut self, s: bool) {
        self.clear_area(
            self.status_coords[0],
            self.status_coords[1],
            self.status_coords[0] + 8,
            self.status_coords[1],
        );
        print!(
            "{}{}{}",
            Goto(self.status_coords[0], self.status_coords[1]),
            Fg(self.colors.status),
            match s {
                false => "Playing",
                true => "Paused",
            }
        );
        self.stdout.flush().unwrap();
    }
    //changes the shuffle status
    pub fn update_shuffle(&mut self, s: bool) {
        self.clear_area(
            self.shuffle_coords[0],
            self.shuffle_coords[1],
            self.shuffle_coords[0] + 5,
            self.shuffle_coords[1],
        );
        print!(
            "{}{}{}",
            Goto(self.shuffle_coords[0], self.shuffle_coords[1]),
            Fg(self.colors.shuffle),
            s
        );
        self.stdout.flush().unwrap();
    }
    //changes the repeat status
    pub fn update_repeat(&mut self, r: Repeat) {
        self.clear_area(
            self.repeat_coords[0],
            self.repeat_coords[1],
            self.repeat_coords[0] + 5,
            self.repeat_coords[1],
        );
        print!(
            "{}{}{}",
            Goto(self.repeat_coords[0], self.repeat_coords[1]),
            Fg(self.colors.repeat),
            r
        );
        self.stdout.flush().unwrap();
    }
    //changes the volume value
    pub fn update_volume(&mut self, v: f32) {
        self.clear_area(
            self.volume_coords[0],
            self.volume_coords[1],
            self.volume_coords[0] + 3,
            self.volume_coords[1],
        );
        print!(
            "{}{}{:.0}",
            Goto(self.volume_coords[0], self.volume_coords[1]),
            Fg(self.colors.volume),
            v * 100.0
        );
        self.stdout.flush().unwrap();
    }
    pub fn update_queue(&mut self, q: &VecDeque<u16>) {
        self.clear_area(
            self.queue_coords[0],
            self.queue_coords[1],
            self.term_size[0] - 1,
            self.queue_coords[1],
        );
        let queue = q.iter().map(|t| format!("{} ", t)).collect::<String>();
        print!(
            "{}{}Queue: {}",
            Goto(self.queue_coords[0], self.queue_coords[1]),
            Fg(self.colors.queue),
            if queue.is_empty() { "empty" } else { &queue }
        );
        self.stdout.flush().unwrap();
    }
    //updates the time bar
    pub fn update_bar(&mut self, duration: Duration, time: Duration) {
        let percent = (time.as_millis() * 100) / duration.as_millis();
        print!("{}", Goto(self.bar_coords[0], self.bar_coords[1]));
        for space in 2..self.term_size[0] {
            let a: u128 = ((space * 100) / self.term_size[0]).into();
            if a <= percent {
                print!("{} {}", Bg(self.colors.bar_bg), RESET_BG);
            } else {
                //print!("{} {}", Bg(self.colors.bar_fg), RESET_BG);
                print!(" ");
            }
        }
        print!(
            "{}{}-{}/{}-",
            Goto((self.term_size[0] / 2) - 6, self.bar_coords[1]),
            termion::color::Fg(self.colors.bar_fg),
            format_time(time),
            format_time(duration)
        );
        self.stdout.flush().unwrap();
    }
    pub fn get_selected(&self) -> u16 {
        self.cursor_pos + self.offset
    }
    pub fn scroll_music_list(&mut self) {
        self.clear_area(3, 2, self.term_size[0] - 1, self.lines + 1);
        for i in 0..self.lines {
            print!("{}", Goto(3, i + 2));
            print!(
                "{}{}.{}",
                RESET_FG,
                (i + 1 + self.offset),
                self.music_list_names[(i + self.offset) as usize]
            );
        }
        self.highlight_playing(self.now_playing);
    }
    pub fn dehighlight_playing(&self) {
        if self.now_playing < self.offset || self.now_playing > (self.lines + self.offset - 1) {
            return;
        }
        print!("{}", Goto(3, self.now_playing - self.offset + 2));
        print!(
            "{}{}.{}  ",
            RESET_FG,
            self.now_playing + 1,
            self.music_list_names[self.now_playing as usize]
        );
    }
    pub fn highlight_playing(&mut self, id: u16) {
        self.dehighlight_playing();
        self.now_playing = id;
        if self.now_playing < self.offset || self.now_playing > (self.lines + self.offset - 1) {
            return;
        }
        print!("{}", Goto(3, self.now_playing - self.offset + 2));
        if self.cursor_pos == self.now_playing - self.offset + 1 {
            print!(
                "{}{}. {}",
                Fg(self.colors.both),
                self.now_playing + 1,
                self.music_list_names[self.now_playing as usize]
            );
        } else {
            print!(
                "{}{}.{}  ",
                Fg(self.colors.now_playing),
                self.now_playing + 1,
                self.music_list_names[self.now_playing as usize]
            );
        };
    }
    pub fn select_track(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        print!("{}", Goto(3, self.cursor_pos + 1));
        if self.now_playing >= self.offset && self.cursor_pos == self.now_playing - self.offset + 1
        {
            print!(
                "{}{}. {}",
                Fg(self.colors.both),
                self.cursor_pos + self.offset,
                self.music_list_names[(self.cursor_pos - 1 + self.offset) as usize]
            );
        } else {
            print!(
                "{}{}. {}",
                Fg(self.colors.selected),
                self.cursor_pos + self.offset,
                self.music_list_names[(self.cursor_pos - 1 + self.offset) as usize]
            );
        }
        self.stdout.flush().unwrap();
    }
    fn deselect_track(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        print!("{}", Goto(3, self.cursor_pos + 1));
        print!(
            "{}{}.{} ",
            RESET_FG,
            self.cursor_pos + self.offset,
            self.music_list_names[(self.cursor_pos - 1 + self.offset) as usize]
        );
        self.highlight_playing(self.now_playing);
    }
    pub fn select_next_track(&mut self) {
        if (self.cursor_pos + self.offset) as usize >= self.music_list_names.len() {
            return;
        }
        self.deselect_track();
        if self.cursor_pos + 1 > self.lines {
            self.offset += 1;
            self.scroll_music_list();
        } else {
            self.cursor_pos += 1;
        }
        self.deselect_track();
        self.select_track();
    }
    pub fn select_previous_track(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }
        self.deselect_track();
        if self.cursor_pos == 1 {
            if self.offset != 0 {
                self.offset -= 1
            };
            self.scroll_music_list();
        } else {
            self.cursor_pos -= 1;
        }
        self.deselect_track();
        self.select_track();
    }
    pub fn highlight_searched(&mut self, search_str: &str) {
        self.deselect_track();
        let search_id = match self
            .music_list_names
            .iter()
            .position(|s| s.to_lowercase().contains(search_str))
        {
            Some(i) => (i + 1) as u16,
            None => self.cursor_pos + self.offset,
        };
        if search_id > self.lines {
            self.offset = search_id - self.lines + 1;
        } else {
            self.offset = 0;
        }
        self.cursor_pos = search_id - self.offset;
        //if self.cursor_pos + self.offset > self.lines + self.offset {
        self.scroll_music_list();
        //}
        //self.display_searching(search_str);
        self.select_track();
    }
    pub fn display_searching(&mut self, s: &str) {
        self.clear_area(
            self.queue_coords[0],
            self.queue_coords[1],
            self.term_size[0] - 1,
            self.queue_coords[1],
        );
        print!("{}", Goto(self.queue_coords[0], self.queue_coords[1]));
        print!("Search: {}", s);
        self.stdout.flush().unwrap();
    }
    //clear a given area
    fn clear_area(&mut self, start_x: u16, start_y: u16, fin_x: u16, fin_y: u16) {
        for line in start_y..fin_y + 1 {
            for column in start_x..fin_x + 1 {
                print!("{} ", Goto(column, line));
            }
        }
        self.stdout.flush().unwrap();
    }
}

fn draw_vertical_lines(term_size: [u16; 2]) {
    print!("{}{}", RESET_FG, RESET_BG);
    for i in 1..term_size[1] + 1 {
        print!("{}|", Goto(1, i));
        print!("{}|", Goto(term_size[0], i));
    }
}
fn draw_horizontal_lines(term_width: u16, char: &str) {
    for _ in 2..term_width + 1 {
        print!("{char}");
    }
}

fn init_music_list(list: &Vec<String>, lines: u16) {
    print!("{}", Goto(3, 2));
    for i in 0..lines {
        print!(
            "{}{}{}.{}",
            Goto(3, i + 2),
            RESET_FG,
            i + 1,
            list[i as usize] //why
        );
    }
}

//inefficiently formats time to hh:mm:ss
fn format_time(dur: Duration) -> String {
    let minutes = (dur.as_secs() / 60) % 60;
    let seconds = dur.as_secs() % 60;
    let hours = (dur.as_secs() / 3600) % 60;
    let mut time_string: String;

    if hours != 0 {
        time_string = hours.to_string();
        time_string.push(':');
        if minutes <= 9 {
            time_string.push('0');
        }
        time_string.push_str(&minutes.to_string());
    } else {
        time_string = minutes.to_string();
    }

    time_string.push(':');
    if seconds <= 9 {
        time_string.push('0');
    }
    time_string.push_str(&seconds.to_string());
    time_string
}
