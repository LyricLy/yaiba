use num_bigint::BigUint;
use num_bigint::ToBigUint;
use std::char;
use std::convert::TryInto;
use std::fmt;
use std::fs;
use std::io::{stdin, BufRead, Read};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Clone, Debug, Eq, PartialEq)]
enum Instruction {
    Set(BigUint),
    Spawn(BigUint),
    SpawnMulti(BigUint),
    Kill(BigUint),
    Jump(BigUint),
    Read,
    Write,
    Halt,
    Sleep,
    GlobalAdd,
    GlobalSubtract,
}
use Instruction::*;

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Set(v) => format!("={}", v),
            Spawn(v) => format!("+{}", v),
            SpawnMulti(v) => format!("*{}", v),
            Kill(v) => format!("-{}", v),
            Jump(v) => format!(":{}", v),
            Read => String::from("r"),
            Write => String::from("w"),
            Halt => String::from("!"),
            Sleep => String::from("."),
            GlobalAdd => String::from("$"),
            GlobalSubtract => String::from("~"),
        };
        write!(f, "{}", s)
    }
}

type Code = Vec<Vec<Instruction>>;

fn parse_single(line: &mut String) -> Option<Instruction> {
    let r = match line.chars().next().unwrap() {
        'r' => Some(Read),
        'w' => Some(Write),
        '!' => Some(Halt),
        '.' => Some(Sleep),
        '$' => Some(GlobalAdd),
        '~' => Some(GlobalSubtract),
        _ => None,
    };
    if r.is_some() {
        line.drain(..1);
    }
    r
}

fn parse_with_value(line: &mut String) -> Option<Instruction> {
    if let Some(constructor) = match line.chars().next().unwrap() {
        '=' => Some(Set as fn(BigUint) -> Instruction),
        '+' => Some(Spawn as fn(BigUint) -> Instruction),
        '*' => Some(SpawnMulti as fn(BigUint) -> Instruction),
        '-' => Some(Kill as fn(BigUint) -> Instruction),
        ':' => Some(Jump as fn(BigUint) -> Instruction),
        _ => None,
    } {
        line.drain(..1);
        let mut n: BigUint = 0.to_biguint().unwrap();
        let mut fired = false;
        loop {
            if line.is_empty() {
                break;
            }
            if let Some(d) = line.chars().next().unwrap().to_digit(10) {
                line.drain(..1);
                n = n * 10.to_biguint().unwrap() + d;
                fired = true;
            } else {
                break;
            }
        }
        if fired {
            return Some(constructor(n));
        } else {
            return None;
        }
    }
    None
}

fn parse_comment(line: &mut String) -> Option<Instruction> {
    if line.starts_with("#") {
        line.clear();
        return None;
    }
    panic!("parse error")
}

fn parse(program: &mut String) -> Code {
    // remove spaces
    program.retain(|c| c != ' ');

    let mut parsed_program = Vec::new();
    for line in program.split("\n") {
        let mut string = line.to_string();
        let mut code = Vec::new();
        while !string.is_empty() {
            for parser in [parse_single, parse_with_value, parse_comment].iter() {
                match parser(&mut string) {
                    Some(i) => {
                        code.push(i);
                        break;
                    }
                    None => {}
                }
            }
        }
        parsed_program.push(code);
    }
    parsed_program
}

#[derive(Clone, Debug)]
struct Thread(BigUint, usize, usize);

fn interpret(code: Code, mut input: Vec<usize>, ascii: bool) {
    let mut threads: Vec<Thread> = vec![Thread(0.to_biguint().unwrap(), 0, 0)];
    let mut new: Vec<Thread> = Vec::new();

    while !threads.is_empty() {
        let mut idx = 0;
        while idx < threads.len() {
            if threads[idx].2 >= code[threads[idx].1].len() {
                threads[idx].2 = 0;
                threads[idx].1 += 1;
                while threads[idx].1 < code.len() && code[threads[idx].1].is_empty() {
                    threads[idx].1 += 1;
                }
                if threads[idx].1 >= code.len() {
                    threads.remove(idx);
                    idx -= 1;
                    continue;
                }
            }

            let inst = &code[threads[idx].1][threads[idx].2];
            threads[idx].2 += 1;

            match inst {
                Set(v) => {
                    threads[idx].0 = v.clone();
                }
                Spawn(l) => {
                    new.push(Thread(
                        0.to_biguint().unwrap(),
                        TryInto::<usize>::try_into(l).unwrap() - 1,
                        0,
                    ));
                }
                SpawnMulti(l) => {
                    let mut count = threads[idx].0.clone();
                    while count > 0.to_biguint().unwrap() {
                        new.push(Thread(
                            0.to_biguint().unwrap(),
                            TryInto::<usize>::try_into(l).unwrap() - 1,
                            0,
                        ));
                        count -= 1.to_biguint().unwrap();
                    }
                }

                Kill(v) => {
                    let mut i = 0;
                    let mut count = 0;
                    let mut idx_drop = 0;
                    threads.retain(|thr: &Thread| {
                        if i == idx || thr.0 != *v {
                            i += 1;
                            true
                        } else {
                            if i <= idx {
                                idx_drop += 1;
                            }
                            i += 1;
                            count += 1;
                            false
                        }
                    });
                    idx -= idx_drop;
                    threads[idx].0 = count.to_biguint().unwrap();
                }

                Jump(l) => {
                    threads[idx].1 = TryInto::<usize>::try_into(l).unwrap() - 1;
                    threads[idx].2 = 0;
                }

                Read => {
                    if input.is_empty() {
                        panic!("interactive input unimplemented");
                    } else {
                        threads[idx].0 = input.drain(..1).next().unwrap().to_biguint().unwrap();
                    }
                }

                Write => {
                    let v = &threads[idx].0;
                    if ascii {
                        const MSG: &'static str =
                            "program tried to output invalid value; try running without --ascii";
                        let c = char::from_u32(v.try_into().expect(MSG)).expect(MSG);
                        print!("{}", c);
                    } else {
                        println!("{}", v);
                    }
                }

                Halt => {
                    threads.remove(idx);
                    if idx == 0 {
                        continue;
                    }
                    idx -= 1;
                }
                Sleep => {}
                GlobalAdd => {
                    let v = threads[idx].0.clone();
                    for (i, thr) in threads.iter_mut().enumerate() {
                        if i != idx {
                            thr.0 += &v;
                        }
                    }
                }
                GlobalSubtract => {
                    let v = threads[idx].0.clone();
                    for (i, thr) in threads.iter_mut().enumerate() {
                        if i != idx {
                            if thr.0 < v {
                                thr.0 = 0.to_biguint().unwrap();
                            } else {
                                thr.0 -= &v;
                            }
                        }
                    }
                }
            }
            idx += 1;
        }

        threads.append(&mut new);
    }
}

#[derive(Debug, StructOpt)]
#[structopt(setting = structopt::clap::AppSettings::TrailingVarArg, name = "ishi", version = "0.1.0", about = "Run a Rui program.")]
struct Opt {
    #[structopt(parse(from_os_str), help = "The program to run.")]
    program: PathBuf,

    #[structopt(
        short,
        long,
        help = "Output characters, treating numbers as Unicode codepoints instead of outputting digits."
    )]
    ascii: bool,

    #[structopt(
        help = "Input to take in the form of space-seperated numbers. When this runs out, the program will prompt for characters from stdin."
    )]
    input: Vec<usize>,
}

fn main() {
    let matches = Opt::from_args();
    let mut program = fs::read_to_string(&matches.program).expect("Invalid file");

    let ast = parse(&mut program);
    interpret(ast, matches.input, matches.ascii);
}
