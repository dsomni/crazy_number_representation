use dashmap::DashMap;
use std::cmp::min;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::Write;
use std::process;
use std::time::Instant;

use std::sync::{Mutex, RwLock};

extern crate dashmap;
extern crate lazy_static;
extern crate num_cpus;

lazy_static::lazy_static! {
    pub static ref BIN_OPERATIONS: RwLock<Mutex<Vec<String>>> = RwLock::new(Mutex::new(Vec::new()));
    pub static ref RES_MAP: DashMap<String, bool> = DashMap::new(); // (calculation res, number of remaining operations)

    pub static ref IS_IN_THREADS: RwLock<Mutex<bool>> = RwLock::new(Mutex::new(false));

}

use threadpool::ThreadPool;

// _ means 'concatenations' here
// ~ means 'unary minus' here
const _BASIC_OPERATIONS: &str = "_ + * - ";
const _OPTIONAL_OPERATIONS: &str = "^ / ";
const _UNARY_OPERATIONS: &str = "~ ";
const _SEP: &str = " ";
const _DIGITS: u8 = 9;

#[derive(Debug)]
enum Operations {
    All,
    AllNoUnary,
    Basic,
}

#[derive(Debug)]
struct Config {
    number: u64,
    step_to_parallel: u8,
    operations_set: Operations,
}

struct Params {
    step: u8,
    available: u8,
    digits_in_row: u8,
    start: Instant,
    step_to_parallel: u8,
}

impl Config {
    fn new() -> Config {
        print!(
            "Please enter config information in one of the following formats:\n
    number(u64)          or\n
    number(u64) step_to_parallel(u8) operations_set(1..3)\n
    where operations_set:\n
        1 -> [_, +, *, -, ^, /, ~]\n
        2 -> [_, +, *, -, ^, /]\n
        3 -> [_, +, *, -]\n"
        );
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Error: unable to read user input");
        let args: Vec<&str> = input.split_whitespace().collect();
        match args.len() {
            3 => Config {
                number: args[0].parse::<u64>().unwrap(),
                step_to_parallel: args[1].parse::<u8>().unwrap(),
                operations_set: match args[2].parse::<u8>().unwrap() {
                    1 => Operations::All,
                    2 => Operations::AllNoUnary,
                    3 => Operations::Basic,
                    _ => panic!("Incorrect operation set chosen"),
                },
            },
            2 => Config {
                number: args[0].parse::<u64>().unwrap(),
                step_to_parallel: args[1].parse::<u8>().unwrap(),
                operations_set: Operations::AllNoUnary,
            },
            1 => Config {
                number: args[0].parse::<u64>().unwrap(),
                step_to_parallel: _DIGITS,
                operations_set: Operations::AllNoUnary,
            },
            _ => panic!("Incorrect input!"),
        }
    }
}

fn decompose(config: Config) {
    let mut ops = String::from(_BASIC_OPERATIONS);
    let start = Instant::now();
    match config.operations_set {
        Operations::All => {
            ops.push_str(_OPTIONAL_OPERATIONS);
            ops.push_str(_UNARY_OPERATIONS);
        }
        Operations::AllNoUnary => {
            ops.push_str(_OPTIONAL_OPERATIONS);
        }
        Operations::Basic => {}
    };
    // let operations: Vec<&str> = ops.trim().split(_SEP).collect();
    // let operations = vec!["_","+","*","^"];
    // println!("{:?}", operations);
    let params = Params {
        step: 3,
        available: _DIGITS - 1,
        digits_in_row: 2,
        start,
        step_to_parallel: config.step_to_parallel,
    };

    {
        *BIN_OPERATIONS.write().unwrap().lock().unwrap() =
            ops.trim().split(_SEP).map(|x| String::from(x)).collect();
    }

    gen_equation(config.number, String::from("1 2"), &params);
}

// (1+2)*(3+4)-5 ^ 6 / (7 - 8) + 9
// 1 2 + 3 4 + * 5 6 ^ - 7 8 - / 9 +

fn gen_equation(number: u64, result: String, params: &Params) {
    //  (8-available)+1 == step-1

    // if (_DIGITS - 1) - params.available + 1 == params.step - 1 {
    //     let key = format!("{}${}", (calculate(result)*1000.0).round()/1000.0, params.step);
    //     if key != "inf" && used.get(&key) != Option::None {
    //         return;
    //     } else {
    //         used.insert(key, true);
    //     }
    // }

    // if (_DIGITS - 1) - params.available + 1 == params.step - 1 {
    //     let key = format!("{}${}", (calculate(result)*1000.0).round()/1000.0, params.step);
    //     if key != "inf" && used.get(&key) != Option::None {
    //         return;
    //     } else {
    //         used.insert(key, true);
    //     }
    // }

    let operations: Vec<String>;
    let is_in_threads: bool;
    {
        operations = BIN_OPERATIONS.read().unwrap().lock().unwrap().to_vec();
        is_in_threads = *IS_IN_THREADS.read().unwrap().lock().unwrap();
    }

    let timer = params.start.elapsed().as_millis();
    if (timer % 10000) == 0 {
        println!("{}\n{}s", result, timer / 1000);
    }
    if params.available == 0 && params.step == (_DIGITS + 1) {
        // println!("{}", result);

        let calculated = calculate(&result);

        // write_to_file(&calculated, result);

        if calculated == number as f64 {
            println!(
                "Answer:\n{}\nFor number:\n{}\nTime: {}ms",
                result, calculated, timer
            );
            process::exit(0);
        }
    }
    if params.step < _DIGITS + 1 {
        let add_number = format!("{} {}", result, params.step);
        let new_params = Params {
            step: params.step + 1,
            digits_in_row: params.digits_in_row + 1,
            ..*params
        };
        gen_equation(number, add_number, &new_params);
    }
    let can_use = min(
        params.available,
        params.step - 2 - (_DIGITS - 1 - params.available),
    );
    if can_use > 0 {
        // println!("{} {}", params.step, params.step_to_parallel);
        if !is_in_threads && params.step == params.step_to_parallel {
            {
                *IS_IN_THREADS.write().unwrap().lock().unwrap() =
                    true;
            }
            let pool = ThreadPool::new(num_cpus::get());
            for operation in operations {
                let add_operation = format!("{} {}", result, operation);
                if operation == "_" {
                    if params.digits_in_row >= 2 {
                        let new_params = Params {
                            available: params.available - 1,
                            digits_in_row: params.digits_in_row - 1,
                            ..*params
                        };
                        pool.execute(move || {
                            gen_equation(number, add_operation, &new_params);
                        });
                    }
                } else {
                    let new_params = Params {
                        available: params.available - 1,
                        digits_in_row: 0,
                        ..*params
                    };
                    pool.execute(move || {
                        gen_equation(number, add_operation, &new_params);
                    });
                }
            }
            pool.join();
            {
                *IS_IN_THREADS.write().unwrap().lock().unwrap() =
                    false;
            }
        } else {
            for operation in operations {
                let add_operation = format!("{} {}", result, operation);
                if operation == "_" {
                    if params.digits_in_row >= 2 {
                        let new_params = Params {
                            available: params.available - 1,
                            digits_in_row: params.digits_in_row - 1,
                            ..*params
                        };
                        gen_equation(number, add_operation, &new_params)
                    }
                } else {
                    let new_params = Params {
                        available: params.available - 1,
                        digits_in_row: 0,
                        ..*params
                    };
                    gen_equation(number, add_operation, &new_params)
                }
            }
        }
    }
}

fn write_to_file(calculated: &f64, result: &str) -> Result<(), io::Error> {
    if *calculated > 0.0 && *calculated < 9223372036854775807.0 && calculated.fract() == 0.0 {
        let mut output = OpenOptions::new()
            .write(true)
            .append(true)
            .open("output.txt")?;
        write!(output, "{} = {}\n", *calculated as i64, result)?;
    }
    Ok(())
}

fn calculate(result: &String) -> f64 {
    evaluate(result).unwrap()
}

fn main() {
    let configs = Config::new();
    // File::create("output.txt");

    // println!("{}", calculate("3 4 5 6 7 8 9 _ _ _ _ - ^"));
    decompose(configs);
    println!("Unreachable");
}

enum Operator {
    Addition,
    Substraction,
    Multiplication,
    Division,
    Concatenation,
    Power,
}

enum OperationElt {
    Operator(Operator),
    Operand(f64),
}

fn tokenizer(expr: &String) -> Result<Vec<OperationElt>, String> {
    expr.split_whitespace()
        .map(|el| match el {
            "+" => Ok(OperationElt::Operator(Operator::Addition)),
            "-" => Ok(OperationElt::Operator(Operator::Substraction)),
            "*" => Ok(OperationElt::Operator(Operator::Multiplication)),
            "/" => Ok(OperationElt::Operator(Operator::Division)),
            "_" => Ok(OperationElt::Operator(Operator::Concatenation)),
            "^" => Ok(OperationElt::Operator(Operator::Power)),
            operand => match operand.parse::<f64>() {
                Ok(val) => Ok(OperationElt::Operand(val)),
                Err(_) => Err(format!("Cannot parse operand \"{}\"", operand)),
            },
        })
        .into_iter()
        .collect()
}

pub fn evaluate(expr: &String) -> Result<f64, String> {
    return match tokenizer(expr) {
        Ok(tokens) => {
            let mut stack: Vec<f64> = Vec::new();
            for token in tokens {
                let mut very_small = false;
                match token {
                    OperationElt::Operator(operator) => {
                        if stack.len() < 2 {
                            return Err("Unsufficient operands before operator".to_string());
                        }
                        let operand2 = stack.pop().expect("expected f64 values in stack");
                        let operand1 = stack.pop().expect("expected f64 values in stack");
                        let result = match operator {
                            Operator::Addition => operand1 + operand2,
                            Operator::Substraction => operand1 - operand2,
                            Operator::Multiplication => operand1 * operand2,
                            Operator::Division => operand1 / operand2,
                            Operator::Concatenation => {
                                let s_o = operand2.to_string();
                                operand1 * 10.0_f64.powi(s_o.len() as i32) + operand2
                            }
                            Operator::Power => operand1.powf(operand2),
                        };
                        stack.push(result);
                    }
                    OperationElt::Operand(val) => stack.push(val),
                }
            }
            if stack.len() != 1 {
                return Err("Remaining untreated operands. Probably missing operator.".to_string());
            }
            return Ok(stack
                .pop()
                .expect("expected a f64 value remaining in stack"));
        }
        Err(err) => Err(err),
    };
}
