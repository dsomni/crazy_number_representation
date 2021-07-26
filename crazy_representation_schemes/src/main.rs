#[macro_use]
extern crate lazy_static;
extern crate num_cpus;
extern crate dashmap;

mod configs_lib;
mod custom_writer;
mod generator;
mod rpn_evaluator;
mod schemes_lib;

fn main() {
    //custom_writer::create_file_to_write("output.txt");

    let configs = configs_lib::Config::new();

    let number = configs.get_number();
    let bin_operations = configs.get_operations();
    let operands = configs_lib::Config::generate_operands(configs_lib::Operands::_DigitsAscending);

    // let schemes = schemes_lib::generate_schemes(operands);
    // generator::generate_threads(schemes, bin_operations, number);

    let schemes = schemes_lib::generate_schemes(operands);
    generator::generate_threads(schemes, bin_operations, number);

    // generator::generate_threads2(operands, bin_operations, number);


    // thread::sleep(Duration::from_secs(5));

    // println!("{}", rpn_evaluator::evaluate("1 2 3 4 5 6 7 _ 8 + / / + 9 / + +").unwrap());
}
