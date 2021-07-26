use std::io;

// _ means 'concatenations' here
// ~ means 'unary minus' here

const _BASIC_OPERATIONS: &str = "_ + * - ";
// const _BASIC_OPERATIONS: &str = "_ ";
const _OPTIONAL_OPERATIONS: &str = "^ / ";
const _UNARY_OPERATIONS: &str = "~ ";
const _SEP: &str = " ";

#[derive(Debug)]
pub enum Operands {
    _DigitsAscending,
    _DigitsDescending,
    _OneToThree,
    _OneToTwo,
    _OneToSix,
}

#[derive(Debug)]
enum Operations {
    All,
    Basic,
}

#[derive(Debug)]
pub struct Config {
    number: u64,
    bin_operations: Operations,
}

impl Config {
    pub fn _show_info() {
        println!(
            "Please enter config information in one of the following formats:\n
    number(u64)          or\n
    number(u64) bin_operations(1..2)\n
    where bin_operations:\n
        1 -> [_, +, -, *, ^, /]\n
        2 -> [_, +, -, *]\n"
        );
    }

    pub fn new() -> Config {
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Error: unable to read user input");
        let args: Vec<&str> = input.split_whitespace().collect();
        match args.len() {
            2 => Config {
                number: args[0].parse::<u64>().unwrap(),
                bin_operations: match args[1].parse::<u8>().unwrap() {
                    1 => Operations::All,
                    2 => Operations::Basic,
                    _ => panic!("Incorrect operation set chosen"),
                },
            },
            1 => Config {
                number: args[0].parse::<u64>().unwrap(),
                bin_operations: Operations::All,
            },
            _ => panic!("Incorrect input!"),
        }
    }

    pub fn get_operations(&self) -> Vec<String> {
        let mut ops = String::from(_BASIC_OPERATIONS);
        match self.bin_operations {
            Operations::All => {
                ops.push_str(_OPTIONAL_OPERATIONS);
            }
            Operations::Basic => {}
        };
        ops.trim().split(_SEP).map(|x| x.to_string()).collect()
    }

    pub fn get_number(&self) -> u64 {
        self.number
    }

    pub fn generate_operands(kind: Operands) -> Vec<String> {
        match kind {
            Operands::_DigitsAscending => (1..10).map(|x| x.to_string()).collect(),
            Operands::_DigitsDescending => (9..-1).map(|x| x.to_string()).collect(),
            Operands::_OneToThree => (1..4).map(|x| x.to_string()).collect(),
            Operands::_OneToTwo => (1..3).map(|x| x.to_string()).collect(),
            Operands::_OneToSix => (1..7).map(|x| x.to_string()).collect(),
        }
    }
}
