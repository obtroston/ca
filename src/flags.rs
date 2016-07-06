extern crate ca;

use ca::types::Cell;

pub enum AutomatonType {
    Cyclic(ca::nb::Neighborhood, u8, u32), // neighborhood, threshold, states
    Life(Vec<Cell>, Vec<Cell>), // survive, birth
}

pub enum InitType {
    Random(Vec<Cell>), // states
    Points(Vec<(usize, usize)>), // coordinates
}

pub struct Options {
    pub automaton_type: AutomatonType,
    pub init_type: InitType,
    pub width: u32,
    pub height: u32,
    pub cell_width: u32,
    pub delay: u32,
}

fn parse_neighborhood(args: &Vec<String>,
                      idx: usize) -> Result<(ca::nb::Neighborhood, usize), &'static str> {
    if args.len() <= idx {
        return Err("Expected neighborhood, found end of args!");
    }
    match &args[idx][..1] {
        c @ "m" | c @ "n" => {
            match (&args[idx][1..]).parse::<u32>() {
                Ok(range) => Ok((
                    match c {
                        "m" => ca::nb::Neighborhood::Moore(range),
                        "n" => ca::nb::Neighborhood::VonNeumann(range),
                        _ => unreachable!(),
                    },
                    idx+1,
                )),
                Err(_) => Err("Neighborhood range must be unsigned 32-bit integer!")
            }
        },
        _ => Err("Neighborhood must start with 'm' or 'n'!"),
    }
}

fn parse_u32(args: &Vec<String>, idx: usize) -> Result<(u32, usize), ()> {
    if args.len() <= idx {
        return Err(());
    }
    match args[idx].parse::<u32>() {
        Ok(val) => Ok((val, idx+1)),
        Err(_) => Err(()),
    }
}

fn parse_cyclic_automaton(args: &Vec<String>,
                          idx: usize) -> Result<(AutomatonType, usize), &'static str> {
    let (nb, idx) = try!(parse_neighborhood(args, idx));
    let (threshold, idx) = try!(
        match parse_u32(args, idx) {
            Ok((val, idx)) => {
                if val > 255 {
                    Err("Threshold must be in range 0..255!")
                } else {
                    Ok((val as u8, idx))
                }
            },
            Err(_) => Err("Invalid or no threshold value!"),
        }
    );
    let (states, idx) = try!(
        match parse_u32(args, idx) {
            Ok((args, idx)) => Ok((args, idx)),
            Err(_) => Err("Invalid or no states count!"),
        }
    );
    Ok((AutomatonType::Cyclic(nb, threshold, states), idx))
}

fn parse_u32_csv(args: &Vec<String>,
                 idx: usize) -> Result<(Vec<u32>, usize), ()> {
    if args.len() <= idx {
        return Err(())
    }
    let mut ints: Vec<u32> = Vec::new();
    for s in args[idx].split(",") {
        match s.parse::<u32>() {
            Ok(val) => ints.push(val),
            Err(_) => return Err(()),
        }
    }
    Ok((ints, idx+1))
}

fn parse_life_automaton(args: &Vec<String>,
                        idx: usize) -> Result<(AutomatonType, usize), &'static str> {
    let (survive, idx) = try!(
        match parse_u32_csv(args, idx) {
            Ok((survive, idx)) => Ok((survive, idx)),
            Err(_) => Err("Invalid SURVIVE_COUNTS value!"),
        }
    );
    let (birth, idx) = try!(
        match parse_u32_csv(args, idx) {
            Ok((birth, idx)) => Ok((birth, idx)),
            Err(_) => Err("Invalid BIRTH_COUNTS value!"),
        }
    );
    Ok((AutomatonType::Life(survive, birth), idx))
}

fn parse_automaton_type(args: &Vec<String>,
                        idx: usize) -> Result<(AutomatonType, usize), &'static str> {
    if args.len() <= idx {
        return Err("Specify automaton type!");
    }
    match &*args[idx] {
        "cyclic" => parse_cyclic_automaton(args, idx+1),
        "life" => parse_life_automaton(args, idx+1),
        _ => Err("Unknown automaton type!"),
    }
}

fn parse_init_state(part: &str) -> Result<(u32, u32), ()> {
    match part.find('*') {
        None => {
            match part.parse::<u32>() {
                Ok(val) => Ok((val, 1)),
                Err(_) => Err(()),
            }
        },
        Some(pos) => {
            match part[..pos].parse::<u32>() {
                Ok(val) => match part[pos+1..].parse::<u32>() {
                    Ok(count) => Ok((val, count)),
                    Err(_) => Err(()),
                },
                Err(_) => Err(()),
            }
        },
    }
}

fn parse_init_random(
    args: &Vec<String>, idx: usize, automaton_type: &AutomatonType
) -> Result<(InitType, usize), &'static str> {
    if args.len() <= idx {
        return Err("INIT_STATES is not set!");
    }
    let states: Vec<u32> = try!(match &*args[idx] {
        "default" => {
            match *automaton_type {
                AutomatonType::Cyclic(_, _, states) => {
                    Ok((0..states).collect())
                },
                AutomatonType::Life(..) => {
                    Ok(vec![0, 1])
                },
            }
        },
        s => {
            let mut states: Vec<u32> = Vec::new();
            for part in s.split(',') {
                let (state, count) = try!(parse_init_state(part)
                                          .map_err(|_| "Invalid INIT_STATES value!"));
                for _ in 0..count {
                    states.push(state);
                }
            }
            Ok(states)
        },
    });
    Ok((InitType::Random(states), idx+1))
}

fn parse_init_points(args: &Vec<String>,
                     idx: usize) -> Result<(InitType, usize), &'static str> {
    if args.len() <= idx {
        return Err("INIT_POINTS is not set!")
    }
    static ERR_MSG: &'static str = "Invalid INIT_POINTS value!";
    let mut points: Vec<(usize, usize)> = Vec::new();
    for part in args[idx].split(';') {
        let points_str: Vec<&str> = part.split(',').collect();
        if points_str.len() != 2 {
            return Err(ERR_MSG);
        }
        let x = try!(points_str[0].parse::<usize>().map_err(|_| ERR_MSG));
        let y = try!(points_str[1].parse::<usize>().map_err(|_| ERR_MSG));
        points.push((x, y));
    }
    Ok((InitType::Points(points), idx+1))
}

fn parse_init_type(
    args: &Vec<String>, idx: usize, automaton_type: &AutomatonType
) -> Result<(InitType, usize), &'static str> {
    if args.len() <= idx {
        return Err("Specify initialization type!");
    }
    match &*args[idx] {
        "random" => parse_init_random(args, idx+1, automaton_type),
        "points" => parse_init_points(args, idx+1),
        _ => Err("Unknown initialization type!"),
    }
}

fn parse_sdl_params(
    args: &Vec<String>, idx: usize
) -> Result<(u32, u32, u32, u32), &'static str> {
    Ok((1920, 1080, 4, 5))
}

pub fn parse_args(args: Vec<String>) -> Result<Options, &'static str> {
    let idx: usize = 1;
    let (automaton_type, idx) = try!(parse_automaton_type(&args, idx));
    let (init_type, idx) = try!(parse_init_type(&args, idx, &automaton_type));
    let (width, height, cell_width, delay) = try!(parse_sdl_params(&args, idx));
    Ok(Options{automaton_type: automaton_type, init_type: init_type,
               width: width, height: height,
               cell_width: cell_width, delay: delay})
}
