extern crate sdl2;

extern crate ca;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

fn draw_automaton(automaton: &ca::CA2, renderer: &mut Renderer, cwidth: u32,
                  palette: &Vec<Color>) {
//    for (state, color) in palette.iter().enumerate() {
//        renderer.set_draw_color(*color);
//        for row in 0..automaton.h {
//            for col in 0..automaton.w {
//                if automaton.cells[row][col] == (state as u32) {
//                    let x = (col as u32)*cwidth;
//                    let y = (row as u32)*cwidth;
//                    renderer.fill_rect(Rect::new(x as i32, y as i32, cwidth, cwidth))
//                        .unwrap();
//                }
//            }
//        }
//    }
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

pub fn main() {
    const WIDTH: u32 = 1280;
    const HEIGHT: u32 = 720;
    const CWIDTH: u32 = 4;
    const AWIDTH: usize = (WIDTH / CWIDTH) as usize;
    const AHEIGHT: usize = (HEIGHT / CWIDTH) as usize;

    let binary_palette = vec![
        Color::RGB(0, 0, 0),
        Color::RGB(200, 200, 0)
    ];
    let octopalette = vec![
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
    let mut timer_subsystem = sdl_context.timer().unwrap();
    let window = video_subsystem.window("test", WIDTH, HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut renderer = window.renderer().build().unwrap();

//    let cells = ca::get_random_area(AWIDTH, AHEIGHT, vec![0,0,0,1]);
//    let mut automaton = ca::CA2::new_life(cells, vec![4,5,6,7,8], vec![3]);
    let cells = ca::get_random_area(AWIDTH, AHEIGHT, vec![0,1,2,3,4,5,6,7,]);
    let mut automaton = ca::CA2::new_cyclic(cells, 2, 5, 8);
//    let cells = ca::get_area_with_points(AWIDTH, AHEIGHT, vec![(AWIDTH/2, AHEIGHT/2)]);
//    let mut automaton = ca::CA2::new_life(cells, vec![1], vec![1]);

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
        draw_automaton(&automaton, &mut renderer, CWIDTH, &octopalette);
        automaton.tick();
        timer_subsystem.delay(5);
    }
}
