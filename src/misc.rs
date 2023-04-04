use std::env;

use crate::{player::{self, Repeat}, sepuku};

fn help() {
    println!("bwmp - badly written music player\r\n
        usage - bwmp -d <dir> <options> \r\n
        Options: \r\n
            -d - specify your music directory, default is current dir\r\n
            -s - toggle shuffle\r\n
            -r <mode>, n - repeat none, s - repeat single, a - repeat all\r\n
            -v <0-200>, volume\r\n
            -h - display this help message\r\n");
}

fn match_repeat(string: &str) -> Option<Repeat> {
    match string {
        "n" => Some(player::Repeat::None),
        "s" => Some(player::Repeat::Single),
        "a" => Some(player::Repeat::All),
        _ => None,
    }
}

fn match_volume(v: &str) -> f32 {
    match v.parse::<f32>() {
        Ok(vol) => vol / 100.0,
        Err(_) => 1.0,
    }
}

//reads arguments and applies them to a new player instance
pub fn construct_player_from_args() -> player::Player {
    let args: Vec<String> = env::args().collect();
    let mut arg_counter = 1;
    let mut repeat = Repeat::None;
    let mut shuffle = false;
    let mut path = ".";
    let mut vol = 1.0;
    while arg_counter < args.len() {
        match args[arg_counter].as_str() {
            "-h" => {help(); sepuku(); std::process::exit(0);},
            "-r" => {
                arg_counter += 1;
                repeat = match_repeat(&args[arg_counter]).unwrap_or_default();
            }
            "-s" => shuffle = true,
            "-v" => {
                arg_counter += 1;
                vol = match_volume(&args[arg_counter]);
            }
            "-d" => {arg_counter += 1; path = &args[arg_counter]},
            _ => (),
        }
        arg_counter += 1;
    }
    player::Player::new(&path, repeat, shuffle, vol)
}

//clears the terminal
pub fn clear_term() {
    let c = std::process::Command::new("clear").output().unwrap();
    println!("{}", String::from_utf8_lossy(&c.stdout));
}

pub fn cut_string(s: String, limit: usize) -> String {
    let new_string = s.replace(".mp3", "");
    if limit >= new_string.len() { return new_string }
    //new_string.chars().take(limit).collect()
    //fucking unicode istg
    //still not perfect, if string has too much japanese characters the name wont fit, but that
    //sound like this person problem, not mine
    let mut char_pos = 0;
    let mut byte_end = 0;
    let mut str_iter = new_string.chars();
    loop {
        if char_pos >= limit {break;}
        if let Some(c) = str_iter.next() {
            char_pos += c.len_utf8();
            byte_end += c.len_utf8();
        } else {break;}
    }
    new_string[0..byte_end].to_string()
}
