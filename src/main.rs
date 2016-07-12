use std::error::Error;
use std::env;

extern crate getopts;
use getopts::{Options};
extern crate sdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

extern crate ca;
mod config;
use config::{CAType, InitType};

static USAGE_TYPE: &'static str = "\
TYPE:
1 RADIUS STATES CODE
  General 1D CA.
  RADIUS: radius of neighborhood, positive non-zero number.
  STATES: count of states, number in range 2-36.
  STATES.pow(2*RADIUS+1) must fit in usize.
  CODE: STATES-base STATES.pow(2*RADIUS+1)-digit number. Far-right number
  sets state of middle cell for neighborhood 0...0, next number to the left
  sets state of middle cell for neighborhood 0...01, ..., far-left number
  sets state of middle cell for neighborhood X...X, where X is last digit in
  base of STATES. Special value 'random' sets random code.

elementary CODE
  Elementary CA.
  CODE: rule code, 0-255.

cyclic NEIGHBORHOOD THRESHOLD STATES
  Cyclic CA.
  NEIGHBORHOOD: mR for Moore neighborhood of range R, nR for Von Neumann
neighborhood of range R.
  THRESHOLD: count of next state neighbors necessary to switch to next
state.
  STATES: count of states.

life SURVIVE BIRTH
  Life-like CA.
  SURVIVE, BIRTH: comma-separated lists of live cells counts needed for
survival/birth. 'empty' stands for empty list.";

fn make_opts() -> Options {
    let mut opts = Options::new();
    opts.optflag(
        "h", "help", "Show this help message."
    );
    opts.optopt(
        "i", "init",
        "(default: random:uniform) World initialization.\n\
        'random' fills cells with random values. STATES: comma-separated list\
        of states or string 'uniform'. Every cell will be randomely filled \
        with one of these states. Instead of writing value V N times you can \
        write V*N. 'uniform' stands for uniform distribution of all possible \
        states. X1,X2,Y1,Y2: if specified, cells will be filled only in this \
        coordinates ranges. For 1D CA values Y1 and Y2 must be omitted.\n\
        'points' fills specified points with value 1 leaving other contain 0. \
        COORDS: semicolon-separated list of coordinates of initially filled \
        cells. For 1D CA coordinate must be integer >= 0, for 2D CA \
        coordinate must have form X,Y, where X and Y are integers >= 0. \
        Special value 'c' means center point. Also you can specify \
        coordinates relative to center point in form c+X/c-X for 1D CA and \
        c+X,Y/c-X,y for 2D CA.",
        "random:STATES[:X1[,X2[,Y1[,Y2]]]] or points:COORDS"
    );
    opts.optopt(
        "s", "size",
        "(default: 2/3 of desktop width and height) Screen size in pixels. \
        Defaults to 2/3 of current desktop width and height.",
        "WIDTHxHEIGHT"
    );
    opts.optopt(
        "c", "cell",
        "(default: maximum divisor of width and height from values 1, 2, 3, \
        4) Cell size in pixels. Must be divisor of width and height.",
        "CELL_WIDTH"
    );
    opts.optopt(
        "d", "delay",
        "(default: 5) Delay after every tick in milliseconds.",
        "DELAY"
    );
    opts
}

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
                  requested_cell_width: Option<u8>) -> Result<u32, String> {
    match requested_cell_width {
        Some(cw) => {
            let cw = cw as u32;
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

fn get_abs_coord(
    origin: usize, shift: i32, limit: usize
) -> Result<usize, &'static str> {
    let abs = (origin as i64) + (shift as i64);
    if abs < 0 || abs >= (limit as i64) {
        return Err("Relative coordinate outside bounds!");
    }
    Ok(abs as usize)
}

fn points1d_to_coords(
    points: Vec<config::Point1D>, ca_width: usize
) -> Result<Vec<usize>, &'static str> {
    let c = ca_width / 2;
    let mut coords: Vec<usize> = Vec::new();
    for p in points {
        let coord = match p {
            config::Point1D::Abs(i) => i,
            config::Point1D::RelToCenter(shift) => {
                try!(get_abs_coord(c, shift, ca_width))
            },
        };
        coords.push(coord);
    }
    Ok(coords)
}

fn points2d_to_coords(
    points: Vec<config::Point2D>, ca_width: usize, ca_height: usize
) -> Result<Vec<(usize, usize)>, &'static str> {
    let c = (ca_width/2, ca_height/2);
    let mut coords: Vec<(usize, usize)> = Vec::new();
    for p in points {
        let coord = match p {
            config::Point2D::Abs(x, y) => (x, y),
            config::Point2D::RelToCenter(x, y) => {(
                try!(get_abs_coord(c.0, x, ca_width)),
                try!(get_abs_coord(c.1, y, ca_height)),
            )},
        };
        coords.push(coord);
    }
    Ok(coords)
}

fn get_ca_view(
    cfg: config::Config, ca_width: usize, ca_height: usize, palette: Vec<Color>
) -> Result<Box<CAView>, String> {
    match cfg.ca_type {
        CAType::Elementary(..) | CAType::CA1{..} => {
            let cells = match cfg.init_type {
                InitType::Random{states, x1, x2, ..} =>
                    ca::gen::random1d(ca_width, states, x1, x2),
                InitType::Points1D(points) => {
                    let coords = try!(points1d_to_coords(points, ca_width));
                    ca::gen::points1d(ca_width, coords)
                },
                _ => unreachable!(),
            };
            let ca = match cfg.ca_type {
                CAType::Elementary(code) =>
                    ca::CA1::new_elementary(cells, code),
                CAType::CA1{radius, states, code} =>
                    try!(ca::CA1::new_ca1(cells, radius, states, code)),
                _ => unreachable!(),
            };
            Ok(Box::new(CA1View::new(ca, palette, ca_height)))
        },
        _ => {
            let cells = match cfg.init_type {
                InitType::Random{states, x1, x2, y1, y2} =>
                    ca::gen::random2d(ca_width, ca_height, states,
                                      x1, x2, y1, y2),
                InitType::Points2D(points) => {
                    let coords = try!(points2d_to_coords(points, ca_width, ca_height));
                    ca::gen::points2d(ca_width, ca_height, coords)
                },
                _ => unreachable!(),
            };
            let ca = match cfg.ca_type {
                CAType::Cyclic(nbh, threshold, states) =>
                    ca::CA2::new_cyclic(cells, nbh, threshold, states),
                CAType::Life(survive, birth) =>
                    ca::CA2::new_life(cells, survive, birth),
                _ => unreachable!(),
            };
            Ok(Box::new(CA2View::new(ca, palette)))
        }
    }
}

fn make_palette() -> Vec<Color> {
    vec![
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
    ]
}

fn print_help(opts: &Options) {
    let short_usage_prefix = format!("{} TYPE", &env::args().nth(0).unwrap());
    let usage_prefix = format!("{}\n\n{}", opts.short_usage(&short_usage_prefix),
                               USAGE_TYPE);
    println!("{}", opts.usage(&usage_prefix))
}

fn execute(opts: &Options) -> Result<(), String> {
    let matches = try!(opts.parse(env::args().skip(1))
                       .map_err(|fail| String::from(fail.description())));
    if matches.opt_present("h") {
        print_help(opts);
        return Ok(());
    }
    let cfg = try!(config::Config::from_matches(&matches));
    let palette = make_palette();
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = try!(make_window(&video_subsystem, cfg.size));
    let (width, height) = window.size();
    let cell_width = try!(get_cell_width(width, height, cfg.cell_width));
    let mut timer_subsystem = sdl_context.timer().unwrap();
    let delay = match cfg.delay { None => 5, Some(d) => d };
    let mut renderer = window.renderer().build().unwrap();
    let ca_width = (width / cell_width) as usize;
    let ca_height = (height / cell_width) as usize;
    let mut ca_view = try!(get_ca_view(cfg, ca_width, ca_height, palette));

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
    Ok(())
}

pub fn main() {
    let opts = make_opts();
    let exit_code = match execute(&opts) {
        Ok(_) => 0,
        Err(s) => {
            println!("{}\nTry -h for more information.", s);
            1
        },
    };
    std::process::exit(exit_code);
}
