extern crate sdl2;

extern crate ca;

mod flags;

use std::env;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

use flags::{AutomatonType, InitType};

const USAGE: &'static str = "
ca TYPE INIT WIDTHxHEIGHT CELL_WIDTH [DELAY]

TYPE:
elementary CODE
cyclic NEIGHBORHOOD THRESHOLD STATES
life SURVIVE_COUNTS BIRTH_COUNTS

'elementary' stands for elementary cellular automaton.
'cyclic' stands for cyclic celular automaton.
'life' stands for life-like automaton.

CODE: code, 0-255.
NEIGHBORHOOD: mR for Moore of range R, nR for Von Neumann of range R.
THRESHOLD: count of next state neighbors necessary to switch to next state.
STATES: count of states.
SURVIVE_COUNTS, BIRTH_COUNTS: comma-separated lists of live cells counts
needed for survival/birth. 'empty' stands for empty list.

INIT:
random [INIT_STATES]
points INIT_POINTS
INIT_STATES: comma-separated list of states or string 'default'. Every cell
will be randomely filled with one of these states. If some value is present in
list more than one time, it will increase probability of filling cell with this
value. Instead of writing value V N times you can write V*N. Omitted value or
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
                  requested_cell_width: Option<u32>) -> Result<u32, String> {
    match requested_cell_width {
        Some(cw) => {
            if width%cw != 0 || height%cw != 0 {
                Err(format!(
                    "Cell width ({}) must me divisor of width ({}) and height ({})!",
                    cw, width, height
                ))
            } else {
                Ok(cw)
            }
        },
        None => {
            Ok((1..5).filter(|x| width%x == 0 && height%x == 0).max().unwrap())
        }
    }
}

trait CAView {
    fn width(&self) -> usize;
    fn height(&self) -> usize;
    fn state_to_color(&self, state: ca::types::Cell) -> Color;
    fn cells(&self) -> &Vec<Vec<ca::types::Cell>>;
    fn tick(&mut self);
}

struct CA1View {
    automaton: ca::CA1,
    cells: Vec<Vec<ca::types::Cell>>,
    palette: Vec<Color>,
    height: usize,
    current_row: usize,
    last_row: usize,
}

impl CA1View {
    fn new(automaton: ca::CA1, palette: Vec<Color>, height: usize) -> CA1View {
        let mut cells = vec![vec![0; automaton.w]; height];
        cells[0].copy_from_slice(&automaton.cells);
        CA1View{automaton: automaton, cells: cells, palette: palette,
                height: height, current_row: 0, last_row: height-1}
    }
}

impl CAView for CA1View {
    fn width(&self) -> usize { self.automaton.w }

    fn height(&self) -> usize { self.height }

    fn state_to_color(&self, state: ca::types::Cell) -> Color {
        self.palette[state as usize]
    }

    fn cells(&self) -> &Vec<Vec<ca::types::Cell>> {
        &self.cells
    }

    fn tick(&mut self) {
        self.automaton.tick();
        if self.current_row < self.last_row {
            self.current_row += 1;
            self.cells[self.current_row].copy_from_slice(&self.automaton.cells);
        } else {
            for row in 0..self.last_row {
                for col in 0..self.automaton.w {
                    self.cells[row][col] = self.cells[row+1][col];
                }
            }
            self.cells[self.last_row].copy_from_slice(&self.automaton.cells);
        }
    }
}

struct CA2View {
    automaton: ca::CA2,
    palette: Vec<Color>,
}

impl CA2View {
    fn new(automaton: ca::CA2, palette: Vec<Color>) -> CA2View {
        CA2View{automaton: automaton, palette: palette}
    }
}

impl CAView for CA2View {
    fn width(&self) -> usize { self.automaton.w }

    fn height(&self) -> usize { self.automaton.h }

    fn state_to_color(&self, state: ca::types::Cell) -> Color {
        self.palette[state as usize]
    }

    fn cells(&self) -> &Vec<Vec<ca::types::Cell>> {
        &self.automaton.cells
    }

    fn tick(&mut self) {
        self.automaton.tick();
    }
}

fn draw_ca(caview: &Box<CAView>, renderer: &mut Renderer, cwidth: u32) {
    for row in 0..caview.height() {
        for col in 0..caview.width() {
            let cell = caview.cells()[row][col];
            let color = caview.state_to_color(cell);
            renderer.set_draw_color(color);
            let x = ((col as u32)*cwidth) as i32;
            let y = ((row as u32)*cwidth) as i32;
            renderer.fill_rect(Rect::new(x, y, cwidth, cwidth)).unwrap();
        }
    }
    renderer.present();
}

fn get_ca_view(options: flags::Options, ca_width: usize, ca_height: usize,
               palette: Vec<Color>) -> Box<CAView> {
    match options.automaton_type {
        AutomatonType::Elementary(code) => {
            let cells = match options.init_type {
                InitType::Random(states) =>
                    ca::gen::random1d(ca_width, states),
                InitType::Points1D(indexes) =>
                    ca::gen::points1d(ca_width, indexes),
                _ => unreachable!(),
            };
            let ca = ca::CA1::new_elementary(cells, code);
            Box::new(CA1View::new(ca, palette, ca_height))
        },
        _ => {
            let cells = match options.init_type {
                InitType::Random(states) =>
                    ca::gen::random2d(ca_width, ca_height, states),
                InitType::Points2D(coords) =>
                    ca::gen::points2d(ca_width, ca_height, coords),
                _ => unreachable!(),
            };
            let ca = match options.automaton_type {
                AutomatonType::Cyclic(nbh, threshold, states) =>
                    ca::CA2::new_cyclic(cells, nbh, threshold, states),
                AutomatonType::Life(survive, birth) =>
                    ca::CA2::new_life(cells, survive, birth),
                _ => unreachable!(),
            };
            Box::new(CA2View::new(ca, palette))
        }
    }
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
    let delay = options.delay;
    let mut renderer = window.renderer().build().unwrap();

    let get_cell_width_result = get_cell_width(width, height, options.cell_width);
    if get_cell_width_result.is_err() {
        println!("{}", get_cell_width_result.unwrap_err());
        return;
    }
    let cell_width = get_cell_width_result.unwrap();
    let ca_width = (width / cell_width) as usize;
    let ca_height = (height / cell_width) as usize;
    let mut ca_view = get_ca_view(options, ca_width, ca_height, palette);

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
        draw_ca(&ca_view, &mut renderer, cell_width);
        ca_view.tick();
        timer_subsystem.delay(delay);
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
