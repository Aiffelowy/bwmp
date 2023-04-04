use termion::color::Rgb;
use std::fs;

pub struct Colors {
    pub name: Rgb,
    pub status: Rgb,
    pub repeat: Rgb,
    pub shuffle: Rgb,
    pub volume: Rgb,
    pub bar_fg: Rgb,
    pub bar_bg: Rgb,
    pub selected: Rgb,
    pub now_playing: Rgb,
    pub both: Rgb,
    pub queue: Rgb,
}

impl Colors {
    pub fn new_default() -> Self {
        Self {
            name: Rgb(255, 255, 0),
            status: Rgb(255, 0, 255),
            repeat: Rgb(255, 0, 0),
            shuffle: Rgb(0, 255, 0),
            volume: Rgb(255, 255, 255),
            bar_fg: Rgb(255, 255, 255),
            bar_bg: Rgb(0, 255, 255),
            selected: Rgb(255, 0, 0),
            now_playing: Rgb(0, 255, 255),
            both: Rgb(255, 0, 255),
            queue: Rgb(0, 0, 0),
        }
    }
    pub fn new_from_config() -> Result<Self, std::io::Error> {
        let mut selfish = Self::new_default();
        let config_path = format!("{}{}", std::env::var("HOME").unwrap(), "/.config/bwmp/config");
        let config_str = fs::read_to_string(config_path)?.replace(" ", "");
        for line in config_str.split("\n") {
            let mut iter = line.split(":");
            match iter.next() {
                Some("name") => selfish.name = parse_rgb(iter.next().unwrap_or_default())?,
                Some("status") => selfish.status = parse_rgb(iter.next().unwrap_or_default())?,
                Some("repeat") => selfish.repeat = parse_rgb(iter.next().unwrap_or_default())?,
                Some("shuffle") => selfish.shuffle =parse_rgb(iter.next().unwrap_or_default())?,
                Some("volume") => selfish.volume = parse_rgb(iter.next().unwrap_or_default())?,
                Some("bar_fg") => selfish.bar_fg = parse_rgb(iter.next().unwrap_or_default())?,
                Some("bar_bg") => selfish.bar_bg = parse_rgb(iter.next().unwrap_or_default())?,
                Some("selected") => selfish.selected = parse_rgb(iter.next().unwrap_or_default())?,
                Some("now_playing") => selfish.now_playing = parse_rgb(iter.next().unwrap_or_default())?,
                Some("both") => selfish.both = parse_rgb(iter.next().unwrap_or_default())?,
                Some("queue") => selfish.queue = parse_rgb(iter.next().unwrap_or_default())?,
                _ => (),
            }
        }
        Ok(selfish)
    }
}

fn parse_rgb(rgb_str: &str) -> Result<Rgb, std::io::Error> {
    let mut rgb :[u8; 3] = [0,0,0];
    let mut t = 0;
    for i in rgb_str.split(",") {
        rgb[t] = i.parse::<u8>().unwrap_or_default();
        t+=1;
    }
    eprint!("{:?}", rgb);
    Ok(Rgb(rgb[0], rgb[1], rgb[2]))
}
