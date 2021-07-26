use dashmap::DashMap;
use std::cmp::min;
use std::process;
use std::sync::{Mutex, RwLock};
use std::thread;
use std::time::Instant;
use threadpool::ThreadPool;

use crate::rpn_evaluator;
use crate::schemes_lib::{generate_schemes, Schema};


const THREADS_COEFFICIENT_4: usize = 16; // 18 30
const THREADS_COEFFICIENT_3: usize = 1; // 18 30
const THREADS_COEFFICIENT_2: usize = 256;
const THREADS_COEFFICIENT: usize = 18; // 18 30

const SHOULD_CHECK_UNIQUENESS: bool = true;
const SHOULD_EXIT_PROCESS: bool = true;

const SHOULD_SHOW_INTERMEDIATE: bool = false;

lazy_static! {
    pub static ref BIN_OPERATIONS: RwLock<Mutex<Vec<String>>> = RwLock::new(Mutex::new(Vec::new()));
    pub static ref NUMBER: RwLock<u64> = RwLock::new(0);
    pub static ref RES_MAP: DashMap<String, bool> = DashMap::new(); // (calculation res, number of remaining operations)

    pub static ref SCHEMES:  Mutex<Vec<Schema>> = Mutex::new(Vec::new());
    pub static ref OPERATIONS_SET: Mutex<Vec<Vec<String>>> = Mutex::new(Vec::new());
}

/**************************************************************************************************************** */
pub fn generate_threads2(operands: Vec<String>, bin_operations: Vec<String>, number: u64) {
    let operands_ln = operands.clone().len() as u32;

    {
        *BIN_OPERATIONS.write().unwrap().lock().unwrap() = bin_operations;

        let mut rw_n = NUMBER.write().unwrap();
        *rw_n = number;
    }

    let scheme_generator_th = thread::spawn(move || {
        generate_schemes2(operands);
    });

    let operations_configs_generator_th = thread::spawn(move || {
        generate_operations(operands_ln as i32 - 1);
    });
    let start = Instant::now();
    println!("Start generate schemes Time {}s", start.elapsed().as_secs());
    for thrd in [scheme_generator_th, operations_configs_generator_th] {
        thrd.join();
    }
    // scheme_generator_th.join();
    // operations_configs_generator_th.join();
    println!("Finish generate schemes Time {}s", start.elapsed().as_secs());
    println!();
    spawn_threads4();
}

pub fn generate_schemes2(operands: Vec<String>) {
    produce_schema(
        (operands.len() - 1) as i32,
        (operands.len() - 1) as i32,
        0,
        Vec::<i32>::new(),
    );
    for result in SCHEMES.lock().unwrap().iter_mut() {
        result.set_result_vec(operands.clone());
    }
}

fn produce_schema(available: i32, max_ops: i32, step: i32, positions: Vec<i32>) {
    if available == 0 && step == (max_ops + 1) {
        SCHEMES.lock().unwrap().push(Schema::from_positions(
            positions.into_iter().rev().collect(),
        ));
        return;
    }

    if step < max_ops + 1 {
        produce_schema(available, max_ops, step + 1, positions.clone());
    }

    let can_use = min(available, step - 2 - (max_ops - 1 - available));
    if can_use > 0 {
        let mut new_positions = positions;
        new_positions.push(step + (max_ops - available));
        produce_schema(available - 1, max_ops, step, new_positions);
    }
}

fn generate_operations(ops_number: i32) {
    generate_op_set(ops_number, Vec::<String>::new());
}

fn generate_op_set(ops_number: i32, ops_vec: Vec<String>) {
    if ops_number == 0 {
        OPERATIONS_SET.lock().unwrap().push(ops_vec.clone());
    } else {
        let ops: Vec<String>;
        {
            ops = BIN_OPERATIONS.read().unwrap().lock().unwrap().to_vec();
        }

        for operation in ops.iter() {
            let mut new_vec = ops_vec.clone();
            new_vec.push(operation.clone());
            generate_op_set(ops_number - 1, new_vec);
        }
    }
}

fn spawn_threads2() {
    let ln = SCHEMES.lock().unwrap().len() * OPERATIONS_SET.lock().unwrap().len();
    let thread_bundle_size = (num_cpus::get() * THREADS_COEFFICIENT_2) as u32;
    println!("Number of threads: {}", ln);
    println!("Number of threads in bundle: {}", thread_bundle_size);
    println!("Number of bundles: {}", ln as u32 / thread_bundle_size);
    drop(ln);

    let mut thrds: Vec<_> = Vec::new();
    let mut i = 0;
    let mut j = 0;
    let start = Instant::now();

    let schemes = &SCHEMES.lock().unwrap().clone();
    let operations_set = &OPERATIONS_SET.lock().unwrap().clone();

    // let schemes: Vec<Schema>;
    // let operations_set : Vec<Vec<String>>;
    // {
    //     schemes = SCHEMES.lock().unwrap().clone().to_vec();
    //     operations_set = OPERATIONS_SET.lock().unwrap().clone().to_vec();
    // }

    for schema in schemes {
        for op_set in operations_set {
            let copy_schema = Schema::from_schema(&schema);
            let copy_op_set = op_set.clone();
            thrds.push(thread::spawn(move || {
                check_and_evaluate(copy_schema, copy_op_set, start);
            }));
            i += 1;
            if i == thread_bundle_size {
                i = 0;
                j += 1;
                println!(
                    "Start Threads Bundle #{}\t Time: {}s",
                    j,
                    start.elapsed().as_secs()
                );
                for thrd in thrds {
                    let _ = thrd.join().unwrap();
                }
                println!(
                    "Finish Threads Bundle #{}\t Time: {}s",
                    j,
                    start.elapsed().as_secs()
                );
                thrds = Vec::new();
            }
        }
    }
    println!(
        "Start Last Threads Bundle \t Time: {}s",
        start.elapsed().as_secs()
    );
    for thrd in thrds {
        let _ = thrd.join().unwrap();
    }
    println!(
        "Finish Last Threads Bundle \t Time: {}s",
        start.elapsed().as_secs()
    );
    println!("Unreachable!")
}

fn spawn_threads3() {
    let ln = OPERATIONS_SET.lock().unwrap().len();
    let thread_bundle_size = (num_cpus::get() * THREADS_COEFFICIENT_2) as u32;
    println!("Number of operations: {}", ln);
    println!("Number of threads in bundle: {}", thread_bundle_size);
    println!("Number of bundles: {}", ln as u32 / thread_bundle_size);
    println!("Threads coefficient: {}", THREADS_COEFFICIENT_2);
    println!();
    drop(ln);

    let mut thrds: Vec<_> = Vec::new();
    let mut i = 0;
    let mut j = 0;
    let start = Instant::now();

    // let schemes = &SCHEMES.lock().unwrap().clone();
    let operations_set = &OPERATIONS_SET.lock().unwrap().clone();

    // let schemes: Vec<Schema>;
    // let operations_set : Vec<Vec<String>>;
    // {
    //     schemes = SCHEMES.lock().unwrap().clone().to_vec();
    //     operations_set = OPERATIONS_SET.lock().unwrap().clone().to_vec();
    // }


    for op_s in operations_set {
        let copy_op_s = op_s.clone();
        thrds.push(thread::spawn(move || {
            check_op_s(copy_op_s, start);
        }));
        i += 1;
        if i == thread_bundle_size {
            i = 0;
            j += 1;
            let st = start.elapsed().as_secs();
            println!(
                "Start Threads Bundle #{}\t Time: {}s",
                j,
                start.elapsed().as_secs()
            );
            for thrd in thrds {
                let _ = thrd.join().unwrap();
            }

            let st2 = start.elapsed().as_secs();
            let mut threads_per_sec: u64;
            if st == st2{
                threads_per_sec = 0;
            }else{
                threads_per_sec = thread_bundle_size as u64 / ( st2 - st);
            }

            println!(
                "Finish Threads Bundle #{}\t Time: {}s Threads per sec: {}",
                j,
                start.elapsed().as_secs(),
                threads_per_sec
            );
            thrds = Vec::new();
        }
    }
    println!(
        "Start Last Threads Bundle \t Time: {}s",
        start.elapsed().as_secs()
    );
    for thrd in thrds {
        let _ = thrd.join().unwrap();
    }
    println!(
        "Finish Last Threads Bundle \t Time: {}s",
        start.elapsed().as_secs()
    );
    println!("Number {} is UNREACHABLE!", NUMBER.read().unwrap())
}

fn spawn_threads4() {
    let ln = OPERATIONS_SET.lock().unwrap().len();
    let thread_bundle_size = (num_cpus::get() * THREADS_COEFFICIENT_3) as u32;
    println!("Number of operations: {}", ln);
    println!("Number of threads in bundle: {}", thread_bundle_size);
    println!("Number of bundles: {}", ln as u32 / thread_bundle_size);
    println!("Threads coefficient: {}", THREADS_COEFFICIENT_3);
    println!();
    drop(ln);

    let start = Instant::now();

    let operations_set = &OPERATIONS_SET.lock().unwrap().clone();

    let pool = ThreadPool::new(thread_bundle_size as usize);

    for op_s in operations_set {
        let copy_op_s = op_s.clone();
        pool.execute(move || {
            check_op_s(copy_op_s, start);
        });
    }
    println!("Start executing Time: {}s", start.elapsed().as_secs());
    pool.join();
    println!("Number {} is UNREACHABLE! Total time: {}s", NUMBER.read().unwrap(), start.elapsed().as_secs());
}

fn check_op_s(op_s: Vec<String>,  start: Instant){
    let schemes = &SCHEMES.lock().unwrap().clone();
    for schema in schemes{
        let copy_op_s = op_s.clone();
        let copy_schema = schema.clone();
        check_and_evaluate(copy_schema, copy_op_s, start);
    }
}

fn check_schema(schema: Schema,  start: Instant){
    let operations_set = &OPERATIONS_SET.lock().unwrap().clone();
    for op_s in operations_set{
        let copy_op_s = op_s.clone();
        let copy_schema = schema.clone();
        check_and_evaluate(copy_schema, copy_op_s, start);
    }
}

fn check_and_evaluate(mut schema: Schema, op_set: Vec<String>, start: Instant) {
    for operator in op_set {
        if !SHOULD_CHECK_UNIQUENESS || check_uniqueness(&schema) {
            match schema.insert_operator(operator) {
                true => (),
                false => {
                    return;
                }
            }
        }
    }

    if !SHOULD_CHECK_UNIQUENESS || check_uniqueness(&schema) {
        let number = NUMBER.read().unwrap();
        let result = &schema.get_string_result();
        if SHOULD_SHOW_INTERMEDIATE {
            println!("{}", result);
        }

        match rpn_evaluator::evaluate(result) {
            Ok(calculated) => {
                if calculated == *number as f64 {
                    println!(
                        "\n{}\n{}\n",
                        result,
                        match rpn_evaluator::to_infix(result) {
                            Ok(x) => x,
                            Err(err) => err,
                        }
                    );
                    println!(
                        "Finish for number {}\nTotal Time: {}s",
                        number,
                        start.elapsed().as_secs()
                    );
                    if SHOULD_EXIT_PROCESS {
                        process::exit(0);
                    }
                }
            }
            Err(error) => {
                if !(error == "Number is too big or too small"
                    || error == "Concatenation is unreal")
                {
                    println!("\nERROR: {}\n", error);
                }
            }
        }
    }
}

/*********************************************************************************************** */

pub fn generate_threads(schemes: Vec<Schema>, bin_operations: Vec<String>, number: u64) {
    let ln = bin_operations.len();
    {
        *BIN_OPERATIONS.write().unwrap().lock().unwrap() = bin_operations;

        let mut rw_n = NUMBER.write().unwrap();
        *rw_n = number;
    }

    // let mut snapshots: Vec<_> = Vec::new();
    // generate_snapshots(&mut snapshots, &schemes, ln);
    // spawn_threads(snapshots);
    spawn_threads_pool(schemes);
}

fn generate_snapshots(snapshots: &mut Vec<(Schema, u16, bool)>, schemes: &Vec<Schema>, ln: usize) {
    for schema in schemes {
        for op_idx in 0..ln {
            let copy_schema = Schema::from_schema(&schema);

            snapshots.push((copy_schema, op_idx as u16, false));
        }
    }
}

fn spawn_threads(args: Vec<(Schema, u16, bool)>) {
    let bundle_size = (num_cpus::get() * THREADS_COEFFICIENT) as u32;
    println!("Number of tasks: {}", args.len());
    println!("Number of threads: {}", bundle_size);
    println!("Number of bundles: {}", args.len() as u32 / bundle_size);

    let mut thrds: Vec<_> = Vec::new();
    let mut i = 0;
    let mut j = 0;
    let start = Instant::now();

    for (schema, op_idx, once_used) in args {
        thrds.push(thread::spawn(move || {
            generate_expression(schema, op_idx, once_used, start);
        }));
        i += 1;
        if i == bundle_size {
            i = 0;
            j += 1;
            println!(
                "Start Threads Bundle #{}\t Time: {}s",
                j,
                start.elapsed().as_secs()
            );
            for thrd in thrds {
                let _ = thrd.join().unwrap();
            }
            println!(
                "Finish Threads Bundle #{}\t Time: {}s",
                j,
                start.elapsed().as_secs()
            );
            thrds = Vec::new();
        }
    }
    println!(
        "Start Last Threads Bundle \t Time: {}s",
        start.elapsed().as_secs()
    );
    for thrd in thrds {
        let _ = thrd.join().unwrap();
    }
    println!(
        "Finish Last Threads Bundle \t Time: {}s",
        start.elapsed().as_secs()
    );
    println!("Unreachable!")
}

fn generate_expression(schema: Schema, op_idx: u16, once_used: bool, start: Instant) {
    let operations = &BIN_OPERATIONS.read().unwrap();
    let number = NUMBER.read().unwrap();
    let remain_operations = schema.get_number_of_remain_positions();

    if !SHOULD_CHECK_UNIQUENESS || check_uniqueness(&schema) {
        if remain_operations == 0 {
            let result = &schema.get_string_result();
            if SHOULD_SHOW_INTERMEDIATE {
                println!("{}", result);
            }

            match rpn_evaluator::evaluate(result) {
                Ok(calculated) => {
                    if calculated == *number as f64 {
                        println!(
                            "\n{}\n{}\n",
                            result,
                            match rpn_evaluator::to_infix(result) {
                                Ok(x) => x,
                                Err(err) => err,
                            }
                        );
                        println!("Finish\nTotal Time: {}s", start.elapsed().as_secs());
                        if SHOULD_EXIT_PROCESS {
                            process::exit(0);
                        }
                    }
                }
                Err(error) => {
                    if !(error == "Number is too big or too small"
                        || error == "Concatenation is unreal")
                    {
                        println!("\nERROR: {}\n", error);
                    }
                }
            }
        } else if remain_operations == 1 && once_used == false {
            let mut copy_schema = Schema::from_schema(&schema);
            if copy_schema.insert_operator(operations.lock().unwrap()[op_idx as usize].clone()) {
                generate_expression(copy_schema, op_idx, true, start);
            }
        } else {
            for idx in 0..op_idx {
                let mut copy_schema = Schema::from_schema(&schema);
                if copy_schema.insert_operator(operations.lock().unwrap()[idx as usize].clone()) {
                    generate_expression(copy_schema, op_idx, once_used, start);
                }
            }
            let mut copy_schema = Schema::from_schema(&schema);
            if copy_schema.insert_operator(operations.lock().unwrap()[op_idx as usize].clone()) {
                generate_expression(copy_schema, op_idx, true, start);
            }
        }
    }
}

fn check_uniqueness(schema: &Schema) -> bool {
    let current_pos = schema.get_current_position();
    let used_positions = schema.get_number_of_used_positions();
    // if used_positions == 2 {
    //     println!("{:?} {} {}", schema, current_pos, used_positions);
    // }

    if current_pos == -1 {
        true
    } else if 2 * used_positions == current_pos as u32 {
        let result = &schema.get_string_result();

        match rpn_evaluator::evaluate(result) {
            Ok(calculated) => {
                let key = format!("{} {}", calculated, used_positions);
                match RES_MAP.get(&key) {
                    None => {
                        RES_MAP.insert(key, true);
                        true
                    }
                    Some(_) => {
                        // println!("{}", result);
                        false
                    }
                }
            }
            Err(_) => true,
        }
    } else {
        true
    }
}

fn spawn_threads_pool(schemes: Vec<Schema>) {
    let start = Instant::now();
    let pool = ThreadPool::new(num_cpus::get()*THREADS_COEFFICIENT_4);
    for schema in schemes {
        pool.execute(move || {
            generate_expression_pool(schema, start);
        });
    }
    pool.join();

    println!("Number {} is UNREACHABLE\nTotal time: {}s\n",  NUMBER.read().unwrap(), start.elapsed().as_secs())
}

fn generate_expression_pool(schema: Schema, start: Instant) {
    let operations = &BIN_OPERATIONS.read().unwrap();
    let number = NUMBER.read().unwrap();
    let remain_operations = schema.get_number_of_remain_positions();
    let op_idx =  BIN_OPERATIONS.read().unwrap().lock().unwrap().len();

    if !SHOULD_CHECK_UNIQUENESS || check_uniqueness(&schema) {
        if remain_operations == 0 {
            let result = &schema.get_string_result();
            if SHOULD_SHOW_INTERMEDIATE {
                println!("{}", result);
            }

            match rpn_evaluator::evaluate(result) {
                Ok(calculated) => {
                    if calculated == *number as f64 {
                        println!(
                            "\n{}\n{}\n",
                            result,
                            match rpn_evaluator::to_infix(result) {
                                Ok(x) => x,
                                Err(err) => err,
                            }
                        );
                        println!("Finish\nTotal Time: {}s", start.elapsed().as_secs());
                        if SHOULD_EXIT_PROCESS {
                            process::exit(0);
                        }
                    }
                }
                Err(error) => {
                    if !(error == "Number is too big or too small"
                        || error == "Concatenation is unreal")
                    {
                        println!("\nERROR: {}\n", error);
                    }
                }
            }
        } else {
            for idx in 0..op_idx {
                let mut copy_schema = Schema::from_schema(&schema);
                if copy_schema.insert_operator(operations.lock().unwrap()[idx as usize].clone()) {
                    generate_expression_pool(copy_schema, start);
                }
            }
        }
    }
}

