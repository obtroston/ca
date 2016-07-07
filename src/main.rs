extern crate sdl2;

extern crate ca;

use std::env;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

mod flags;

const USAGE: &'static str = "
ca TYPE INIT WIDTHxHEIGHT CELL_WIDTH [DELAY]

TYPE:
cyclic NEIGHBORHOOD THRESHOLD STATES
life SURVIVE_COUNTS BIRTH_COUNTS

cyclic stands for cyclic celular automata.
life stands for Conway's Game of Life-like automata.

NEIGHBORHOOD: mR for Moore of range R, nR for Von Neumann of range R.
THRESHOLD: count of next state neighbors necessary to switch to next state.
STATES: count of states.
SURVIVE_COUNTS, BIRTH_COUNTS: comma-separated lists of live cells counts
needed for survival/birth. 'empty' stands for empty list.

INIT:
random [INIT_STATES]
points INIT_POINTS
INIT_STATES: comma-separated list of states. Every cell will be randomely
filled with one of these states. If some value is present in list more than
one time, it will increase probability of filling cell with this value.
Instead of writing value V N times you can write V*N. Omitted value or
'default' stands for uniform distribution of all possible states.
INIT_POINTS: semicolon-separated list of coordinates of initially filled cells
in form x,y.

WIDTH, HEIGHT, CELL_WIDTH, DELAY: unsigned 32-bit integers.
WIDTH, HEIGHT: screen dimensions.
CELL_WIDTH: width of cell.
Following must hold: WIDTH%CELL_WIDTH == HEIGHT%CELL_WIDTH == 0.
DELAY: delay in milliseconds after each tick, defaults to 5.
";

fn make_window(
    video_subsystem: &sdl2::VideoSubsystem, size: Option<(u32, u32)>
) -> Result<sdl2::video::Window, &'static str> {
    let mut window = try!(video_subsystem.window("CA", 0, 0).position(0, 0).build()
                          .map_err(|_| "Failed to create window!"));
    let di = try!(window.display_index().map_err(|_| "Failed to get display index!"));
    let mut dm = try!(video_subsystem.desktop_display_mode(di)
                      .map_err(|_| "Failed to get desktop display mode!"));
    dm.w = ((dm.w as f64) * 2.0 / 3.0) as i32;
    dm.h = ((dm.h as f64) * 2.0 / 3.0) as i32;
    match size {
        Some((w, h)) => {
            dm.w = w as i32;
            dm.h = h as i32;
            dm = try!(video_subsystem.closest_display_mode(di, &dm)
                      .map_err(|_| "Invalid size requested!"));
        },
        _ => (),
    };
    try!(window.set_size(dm.w as u32, dm.h as u32)
         .map_err(|_| "Failed to set window size!"));
    window.show();
    Ok(window)
}

fn get_cell_width(width: u32, height: u32,
                  requested_cell_width: Option<u32>) -> Result<u32, &'static str> {
    match requested_cell_width {
        Some(cw) => {
            if cw%width != 0 || cw%height != 0 {
                Err("Cell width must me divisor of width and height!")
            } else {
                Ok(cw)
            }
        },
        None => {
            Ok((1..5).filter(|x| width%x == 0 && height%x == 0).max().unwrap())
        }
    }
}

fn draw_automaton(automaton: &ca::CA2, renderer: &mut Renderer, cwidth: u32,
                  palette: &Vec<Color>) {
    for row in 0..automaton.h {
        for col in 0..automaton.w {
            renderer.set_draw_color(palette[automaton.cells[row][col] as usize]);
            let x = ((col as u32)*cwidth) as i32;
            let y = ((row as u32)*cwidth) as i32;
            renderer.fill_rect(Rect::new(x, y, cwidth, cwidth))
                .unwrap();
        }
    }
    renderer.present();
}

fn execute(options: flags::Options) {
    let palette = vec![
        Color::RGB(0, 0, 0),
        Color::RGB(200, 200, 0),
	    Color::RGB(0, 153, 255),
	    Color::RGB(0, 255, 153),
	    Color::RGB(51, 255, 0),
	    Color::RGB(255, 255, 0),
	    Color::RGB(255, 51, 0),
	    Color::RGB(255, 0, 153),
	    Color::RGB(182, 0, 255),
	    Color::RGB(37, 0, 255),
	    Color::RGB(0, 102, 255),
	    Color::RGB(0, 255, 204),
	    Color::RGB(0, 255, 0),
	    Color::RGB(204, 255, 0),
	    Color::RGB(255, 102, 0),
	    Color::RGB(255, 0, 102),
	    Color::RGB(219, 0, 255),
	    Color::RGB(73, 0, 255),
    ];

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let make_window_result = make_window(&video_subsystem, options.size);
    if make_window_result.is_err() {
        println!("{}", make_window_result.err().unwrap());
        return;
    }
    let window = make_window_result.unwrap();
    let (width, height) = window.size();
    let mut timer_subsystem = sdl_context.timer().unwrap();
    let mut renderer = window.renderer().build().unwrap();

    let get_cell_width_result = get_cell_width(width, height, options.cell_width);
    if get_cell_width_result.is_err() {
        println!("{}", get_cell_width_result.unwrap_err());
        return;
    }
    let cell_width = get_cell_width_result.unwrap();

    let automaton_width = (width / cell_width) as usize;
    let automaton_height = (height / cell_width) as usize;
    let cells = match options.init_type {
        flags::InitType::Random(states) =>
            ca::gen::random_area(automaton_width, automaton_height, states),
        flags::InitType::Points(coords) =>
            ca::gen::area_with_points(automaton_width, automaton_height, coords),
    };
    let mut automaton = match options.automaton_type {
        flags::AutomatonType::Cyclic(nbh, threshold, states) =>
            ca::CA2::new_cyclic(cells, nbh, threshold, states),
        flags::AutomatonType::Life(survive, birth) =>
            ca::CA2::new_life(cells, survive, birth),
    };

    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                    | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running
                },
                _ => {}
            }
        }
        draw_automaton(&automaton, &mut renderer, cell_width, &palette);
        automaton.tick();
        timer_subsystem.delay(options.delay);
    }
}

pub fn main() {
    match flags::parse_args(env::args().collect()) {
        Ok(options) => execute(options),
        Err(msg) => {
            print!("{}\n{}", msg, USAGE);
        },
    }
}
