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
needed for survival/birth.

INIT:
random INIT_STATES
points INIT_POINTS
INIT_STATES: comma-separated list of states. Every cell will be randomely
filled with one of these states. If some value is present in list more than
one time, it will increase probability of filling cell with this value.
Instead of writing value V N times you can write V*N.
'default' means uniform distribution of all possible states.
INIT_POINTS: semicolon-separated list of coordinates of initially filled cells
in form x,y.

WIDTH, HEIGHT, DELAY: unsigned 32-bit integers.
WIDTH, HEIGHT: screen dimensions.
CELL_WIDTH: width of cell.
Following must hold: WIDTH%CELL_WIDTH == HEIGHT%CELL_WIDTH == 0.
DELAY: delay in milliseconds after each tick, defaults to 0.
";

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
    let automaton_width = (options.width / options.cell_width) as usize;
    let automaton_height = (options.height / options.cell_width) as usize;

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

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let mut timer_subsystem = sdl_context.timer().unwrap();
    let window = video_subsystem.window("CA", options.width, options.height)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut renderer = window.renderer().build().unwrap();

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
        draw_automaton(&automaton, &mut renderer, options.cell_width, &palette);
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
