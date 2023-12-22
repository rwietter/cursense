extern crate ncurses;
extern crate sysinfo;

use ncurses as nc;
use std::thread;
use std::time::Duration;
use sysinfo::System;

#[derive(Debug)]
struct CPUInfo<'a> {
    name: String,
    brand: &'a str,
    frequency: u64,
}

#[derive(Debug)]
struct Config<'a> {
    cpu_usage: f32,
    available_memory: f32,
    total_memory: f32,
    used_memory: f32,
    cores: i8,
    cpus: Vec<CPUInfo<'a>>,
}

impl<'a> Config<'a> {
    fn new(sys: &'a mut System) -> Config {
        sys.refresh_all();
        let cpu_usage = sys.global_cpu_info().cpu_usage() as f32;
        let available_memory = sys.available_memory() as f32;
        let used_memory = sys.used_memory() as f32;
        let total_memory = sys.total_memory() as f32;
        let cores = sys.physical_core_count().unwrap() as i8;
        let cpus = sys.cpus();

        let cpu_infos = cpus.iter().map(|cpu| {
            let name = cpu.name().to_string();
            let brand = cpu.brand();
            let frequency = cpu.frequency();

            CPUInfo {
                name,
                brand,
                frequency,
            }
        });

        Config {
            cpu_usage,
            available_memory: available_memory / 1024.0 / 1024.0,
            used_memory: used_memory / 1024.0 / 1024.0,
            total_memory: total_memory / 1024.0 / 1024.0,
            cpus: cpu_infos.collect(),
            cores,
        }
    }
}

fn main() {
    nc::initscr();
    nc::raw();
    nc::curs_set(nc::CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    nc::keypad(nc::stdscr(), true);
    nc::noecho();
    nc::start_color();
    nc::use_default_colors();
    let win = nc::newwin(10, 30, 2, 2);
    nc::scrollok(win, true);
    nc::init_pair(1, nc::COLOR_RED, nc::COLOR_BLACK);
    nc::init_pair(2, nc::COLOR_CYAN, nc::COLOR_BLACK);
    nc::init_pair(3, nc::COLOR_YELLOW, nc::COLOR_BLACK);
    nc::timeout(0);
    nc::scrollok(nc::stdscr(), true);
    nc::nodelay(nc::stdscr(), true);
    nc::clear();

    let mut sys = System::new();

    loop {
        sys.refresh_all();
        let config = Config::new(&mut sys);

        draw_progress_bar(
            config.available_memory,
            config.total_memory,
            config.used_memory,
        );
        draw_cpu_usage(config.cpu_usage, config.cores);
        draw_cpu_info(config.cpus);

        nc::refresh();

        if nc::getch() == 'q' as i32 {
            break;
        }
        thread::sleep(Duration::from_secs(1));
    }

    nc::endwin();
}

fn draw_cpu_info(cpus: Vec<CPUInfo>) {
    let mut y = 8;
    for cpu in cpus {
        nc::mvprintw(y, 0, &format!("Name: {}", cpu.name));
        nc::mvprintw(y + 1, 0, &format!("Brand: {}", cpu.brand));
        nc::mvprintw(y + 2, 0, &format!("Frequency: {} MHz", cpu.frequency));
        y += 4;
    }
}

fn draw_cpu_usage(cpu_usage: f32, cores: i8) {
    let height = 3; // Altura da barra de progresso
    let width = 20; // Largura da barra de progresso
    let fill_char = '*';

    nc::mvprintw(3, 0, "CPU Usage ");
    let fill_width = ((cpu_usage as f64 * (width - 2) as f64) / 100.0) as u128; // Subtrai 2 para descontar as bordas

    for y in 4..height + 2 {
        for x in 1..fill_width + 1 {
            nc::mvaddch(y as i32, x as i32, fill_char as nc::chtype);
        }
    }

    nc::mvaddch(4, 0, '[' as nc::chtype | nc::A_BOLD());
    nc::mvaddch(4, width as i32 - 1, ']' as nc::chtype | nc::A_BOLD());
    nc::mvprintw(4, 21, &format!("{:.2}%%", cpu_usage));
    nc::mvprintw(6, 0, &format!("({}) Cores", cores));
}

fn draw_progress_bar(available_memory: f32, total_memory: f32, used_memory: f32) {
    let height = 3;
    let width = 20;
    let fill_char = '*';

    nc::mvprintw(0, 0, "Used Memory");
    let fill_width = used_memory as u128 * width as u128 / total_memory as u128;

    for y in 1..height - 1 {
        for x in 1..fill_width + 1 {
            nc::attron(nc::COLOR_PAIR(1));
            nc::mvaddch(y as i32, x as i32, fill_char as nc::chtype);
        }
    }
    nc::attroff(nc::COLOR_PAIR(2));

    nc::attron(nc::COLOR_PAIR(2));
    nc::mvaddch(1, 0, '[' as nc::chtype | nc::A_BOLD());
    nc::mvaddch(1, width as i32 - 1, ']' as nc::chtype | nc::A_BOLD());
    nc::attroff(nc::COLOR_PAIR(2));

    nc::attron(nc::COLOR_PAIR(3));
    nc::mvprintw(
        1,
        21,
        &format!("{:.0} MBs of {:.0} MBs", used_memory, total_memory),
    );
    nc::mvprintw(2, 21, &format!("{:.0} MBs available", available_memory));
    nc::attroff(nc::COLOR_PAIR(3));
}
