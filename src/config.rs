use std::str::FromStr;

extern crate getopts;
use getopts::Matches;

extern crate ca;
use ca::types::Cell;

const ERR_INVALID_RANDOM: &'static str = "Invalid 'random' parameters!";
const ERR_NO_STATES: &'static str = "STATES is not set!";
const ERR_INVALID_STATES: &'static str = "Invalid STATES value!";
const ERR_NO_POINTS: &'static str = "POINTS is not set!";
const ERR_INVALID_POINTS: &'static str = "Invalid POINTS value!";

pub enum CAType {
    CA1{radius: u8, states: u8, code: Option<String>},
    Elementary(u8), // code
    Cyclic(ca::nb::Neighborhood, u8, u32), // neighborhood, threshold, states
    Life(Vec<Cell>, Vec<Cell>), // survive, birth
}

pub enum InitType {
    Random{states: Vec<Cell>,
           x1: Option<usize>, x2: Option<usize>,
           y1: Option<usize>, y2: Option<usize>},
    Points1D(Vec<usize>), // indexes
    Points2D(Vec<(usize, usize)>), // coordinates
}

pub struct Config {
    pub ca_type: CAType,
    pub init_type: InitType,
    pub size: Option<(u32, u32)>,
    pub cell_width: Option<u8>,
    pub delay: Option<u32>,
}

impl Config {
    pub fn from_matches(matches: &Matches) -> Result<Config, &'static str> {
        let ca_type = try!(parse_ca_type(&matches.free));
        let init_type = try!(parse_init_type(matches.opt_str("init"), &ca_type));
        let size = try!(parse_size(matches.opt_str("size")));
        let cell_width = try!(
            match matches.opt_str("cell") {
                Some(s) => {
                    match s.parse::<u8>() {
                        Ok(x) => Ok(Some(x)),
                        Err(_) => Err("Cell width must be unsigned 8-bit integer!"),
                    }
                },
                None => Ok(None),
            }
        );
        let delay = try!(
            match matches.opt_str("delay") {
                Some(s) => {
                    match s.parse::<u32>() {
                        Ok(x) => Ok(Some(x)),
                        Err(_) => Err("Delay must be unsigned 32-bit integer!"),
                    }
                },
                None => Ok(None),
            }
        );
        Ok(Config{ca_type: ca_type, init_type: init_type,
                  size: size, cell_width: cell_width, delay: delay})
    }
}

fn parse<F>(args: &Vec<String>, idx: usize) -> Result<(F, usize), ()>
    where F: FromStr {
    if args.len() <= idx {
        return Err(());
    }
    match args[idx].parse::<F>() {
        Ok(val) => Ok((val, idx+1)),
        Err(_) => Err(()),
    }
}

fn parse_ca1(
    args: &Vec<String>, idx: usize
) -> Result<(CAType, usize), &'static str> {
    let (radius, idx) = try!(
        parse::<u8>(args, idx)
        .map_err(|_| "RADIUS must be unsigned 8-bit integer!")
    );
    let (states, idx) = try!(
        parse::<u8>(args, idx)
        .map_err(|_| "STATES must be unsigned 8-bit integer!")
    );
    if args.len() < idx {
        return Err("Specify CODE value!");
    }
    let code = if args[idx] == "random" { None }
               else { Some(args[idx].clone()) };
    Ok((CAType::CA1{radius: radius, states: states,
                    code: code}, idx+1))
}

fn parse_elementary_ca(
    args: &Vec<String>, idx: usize
) -> Result<(CAType, usize), &'static str> {
    let (code, idx) = try!(
        parse::<u8>(args, idx)
        .map_err(|_| "CODE must be unsigned 8-bit integer!")
    );
    Ok((CAType::Elementary(code as u8), idx))
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

fn parse_cyclic_ca(
    args: &Vec<String>, idx: usize
) -> Result<(CAType, usize), &'static str> {
    let (nb, idx) = try!(parse_neighborhood(args, idx));
    let (threshold, idx) = try!(
        match parse::<u8>(args, idx) {
            Ok((val, idx)) => Ok((val, idx)),
            Err(_) => Err("THRESHOLD must be unsigned 8-bit integer!"),
        }
    );
    let (states, idx) = try!(
        match parse::<u32>(args, idx) {
            Ok((args, idx)) => Ok((args, idx)),
            Err(_) => Err("STATES must be unsigned 32-bit integer!"),
        }
    );
    Ok((CAType::Cyclic(nb, threshold, states), idx))
}

fn parse_u32_csv(s: &str, sep: char) -> Result<Vec<u32>, ()> {
    if s == "empty" {
        return Ok(Vec::new());
    }
    let mut ints: Vec<u32> = Vec::new();
    for part in s.split(sep) {
        match part.parse::<u32>() {
            Ok(x) => ints.push(x),
            Err(_) => return Err(()),
        }
    }
    Ok(ints)
}

fn parse_life_ca(
    args: &Vec<String>, idx: usize
) -> Result<(CAType, usize), &'static str> {
    if args.len() <= idx {
        return Err("SURVIVE is not set!");
    }
    let (survive, idx) = try!(
        match parse_u32_csv(&args[idx], ',') {
            Ok(survive) => Ok((survive, idx+1)),
            Err(_) => Err("Invalid SURVIVE value!"),
        }
    );
    if args.len() <= idx {
        return Err("BIRTH is not set!");
    }
    let (birth, idx) = try!(
        match parse_u32_csv(&args[idx], ',') {
            Ok(birth) => Ok((birth, idx+1)),
            Err(_) => Err("Invalid BIRTH value!"),
        }
    );
    Ok((CAType::Life(survive, birth), idx))
}

fn parse_ca_type(args: &Vec<String>) -> Result<CAType, &'static str> {
    if args.len() <= 0 {
        return Err("Specify CA type!");
    }
    let (ca_type, idx) = try!(
        match &*args[0] {
            "1" => parse_ca1(args, 1),
            "elementary" => parse_elementary_ca(args, 1),
            "cyclic" => parse_cyclic_ca(args, 1),
            "life" => parse_life_ca(args, 1),
            _ => Err("Unknown CA type!"),
        }
    );
    if idx < args.len() {
        Err("Trailing args!")
    } else {
        Ok(ca_type)
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
    s: &str, ca_type: &CAType
) -> Result<InitType, &'static str> {
    if s == "" { return Err(ERR_NO_STATES); }
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() > 2 { return Err(ERR_INVALID_RANDOM); }

    let states = if parts[0] == "uniform" {
        match *ca_type {
            CAType::Cyclic(_, _, states) => { (0..states).collect() },
            _ => { vec![0, 1] },
        }
    } else {
        let mut states = Vec::new();
        for part in s.split(',') {
            let (state, count) = try!(parse_init_state(part)
                                      .map_err(|_| ERR_INVALID_STATES));
            for _ in 0..count {
                states.push(state);
            }
        }
        states
    };

    let (x1, x2, y1, y2) = if parts.len() == 1 {
        (None, None, None, None)
    } else {
        let parts: Vec<&str> = parts[1].split(',').collect();
        let x1 = Some(try!(parts[0].parse::<usize>()
                                   .map_err(|_| "random: invalid X1 value!")));
        let x2 = if parts.len() < 2 {
            None
        } else {
            Some(try!(parts[1].parse::<usize>()
                              .map_err(|_| "random: invalid X2 value!")))
        };
        let y1 = if parts.len() < 3 {
            None
        } else {
            Some(try!(parts[2].parse::<usize>()
                              .map_err(|_| "random: invalid Y1 value!")))
        };
        let y2 = if parts.len() < 4 {
            None
        } else {
            Some(try!(parts[3].parse::<usize>()
                              .map_err(|_| "random: invalid Y2 value!")))
        };
        (x1, x2, y1, y2)
    };

    match *ca_type {
        CAType::Elementary(..) if y1.is_some() || y2.is_some() => {
            return Err("random: Y1 and Y2 values are disabled for 1D CA!");
        },
        _ => (),
    }

    Ok(InitType::Random{states: states, x1: x1, x2: x2, y1: y1, y2: y2})
}

fn parse_points1d(s: &str) -> Result<InitType, ()> {
    let indexes = try!(parse_u32_csv(s, ';'))
                  .iter().map(|x| *x as usize).collect();
    Ok(InitType::Points1D(indexes))
}

fn parse_points2d(s: &str) -> Result<InitType, ()> {
    let mut points: Vec<(usize, usize)> = Vec::new();
    for part in s.split(';') {
        let point_str: Vec<&str> = part.split(',').collect();
        if point_str.len() != 2 {
            return Err(());
        }
        let x = try!(point_str[0].parse::<usize>().map_err(|_| ()));
        let y = try!(point_str[1].parse::<usize>().map_err(|_| ()));
        points.push((x, y));
    }
    Ok(InitType::Points2D(points))
}

fn parse_init_points(
    s: &str, ca_type: &CAType
) -> Result<InitType, &'static str> {
    if s == "" {
        return Err(ERR_NO_POINTS);
    }
    (match *ca_type {
        CAType::Elementary(..) => parse_points1d(s),
        _ => parse_points2d(s),
    }).map_err(|_| ERR_INVALID_POINTS)
}

fn parse_init_type(
    option_value: Option<String>, ca_type: &CAType
) -> Result<InitType, &'static str> {
    static RANDOM_PREFIX: &'static str = "random:";
    static POINTS_PREFIX: &'static str = "points:";
    match option_value {
        None => {
            parse_init_type(Some(format!("{}uniform", RANDOM_PREFIX)), ca_type)
        },
        Some(s) => {
            if s.starts_with(RANDOM_PREFIX) {
                parse_init_random(&s[RANDOM_PREFIX.len()..], ca_type)
            } else if s.starts_with("points:") {
                parse_init_points(&s[POINTS_PREFIX.len()..], ca_type)
            } else {
                Err("Unknown initialization type!")
            }
        }
    }
}

fn parse_size(
    option_val: Option<String>
) -> Result<Option<(u32, u32)>, &'static str> {
    match option_val {
        Some(s) => {
            let xpos = try!(s.find('x')
                            .ok_or("Specify size as WIDTHxHEIGHT!"));
            let w = try!(s[..xpos].parse::<u32>().map_err(|_| "Invalid width!"));
            let h = try!(s[xpos+1..].parse::<u32>().map_err(|_| "Invalid height!"));
            Ok(Some((w, h)))
        },
        None => Ok(None),
    }
}
