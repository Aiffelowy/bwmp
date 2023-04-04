use std::io::stdout;
use std::time::Instant;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

mod colors;
mod misc;
mod player;
mod ui;

fn sepuku() {
    misc::clear_term();
    print!("{}", termion::cursor::Show);
}

fn main() {
    //temp

    //probably permanent
    let _stdout = stdout().into_raw_mode().unwrap();
    let mut player: player::Player = misc::construct_player_from_args();
    let mut search_mode: bool = false;
    let mut search_string = String::new();
    print!("{}", termion::cursor::Hide);
    let mut stdin = termion::async_stdin().keys();
    player.play(0);
    loop {
        let now = Instant::now();
        let input = stdin.next();
        if let Some(Ok(key)) = input {
            if !search_mode {
                match key {
                    Key::Char(' ') => player.toggle_pause(),
                    Key::Char('+') => player.change_volume(player.volume() + 0.1),
                    Key::Char('-') => player.change_volume(player.volume() - 0.1),
                    Key::Char('n') => player.play_next(true),
                    Key::Char('p') => player.play_previous(),
                    Key::Char('r') => player.toggle_repeat(None),
                    Key::Char('s') => player.toggle_shuffle(),
                    Key::Char('\n') => player.play_selected(),
                    Key::Char('a') => player.put_selected_in_queue(),
                    Key::Char('/') => {player.ui.display_searching(""); search_mode = true},
                    Key::Up => player.ui.select_previous_track(),
                    Key::Down => player.ui.select_next_track(),
                    Key::Char('q') => break,
                    _ => (),
                }
            } else {
                match key {
                    Key::Char('\n') | Key::Esc => {
                        search_string.clear();
                        player.ui.update_queue(&player.queue);
                        search_mode = false
                    }
                    Key::Char(char) => {
                        search_string.push(char);
                        //update search stuff on screen
                        player.search_for(&search_string);
                    }
                    Key::Backspace => {
                        search_string.pop();
                        player.search_for(&search_string);
                    }
                    _ => (),
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
        if player.is_finished() {
            player.play_next(false);
        }
        if player.is_playing() {
            player.update_bar(now.elapsed());
        }
    }
    sepuku();
}
