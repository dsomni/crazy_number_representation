use std::cmp::min;

#[derive(Debug, Clone)]
pub struct Schema {
    pub result_vec: Vec<String>,
    pub positions: Vec<i32>,
    pub position_pointer: i16,
    pub prev_not_concat_op_pos: i16,
}

impl Schema {
    fn _new() -> Schema {
        Schema {
            result_vec: Vec::<String>::new(),
            positions: Vec::<i32>::new(),
            position_pointer: -1,
            prev_not_concat_op_pos: -1
        }
    }

    pub fn _from_operands(operands: Vec<String>) -> Schema {
        Schema {
            result_vec: operands.clone(),
            positions: Vec::<i32>::new(),
            position_pointer: -1,
            prev_not_concat_op_pos: -1
        }
    }

    pub fn from_positions(positions: Vec<i32>) -> Schema {
        Schema {
            result_vec: Vec::<String>::new(),
            positions: positions.clone(),
            position_pointer: positions.len() as i16 - 1,
            prev_not_concat_op_pos: -1
        }
    }

    pub fn from_schema(schema: &Schema) -> Schema {
        Schema {
            result_vec: schema.result_vec.clone(),
            positions: schema.positions.clone(),
            position_pointer: schema.position_pointer,
            prev_not_concat_op_pos: schema.prev_not_concat_op_pos,
        }
    }

    pub fn set_result_vec(&mut self, result: Vec<String>) {
        self.result_vec = result;
    }

    pub fn get_string_result(&self) -> String {
        let mut answer = String::from("");
        for symbol in self.result_vec.iter() {
            answer.push_str(symbol);
            answer.push_str(" ")
        }
        answer
    }

    fn _add_position(&mut self, position: i32) {
        self.positions.push(position);
        self.position_pointer += 1;
    }

    pub fn insert_operator(&mut self, operator: String) -> bool {
        if Schema::check_conditions(self, &operator) {
            // if true {
            return match self.positions.get(self.position_pointer as usize) {
                None => false,
                Some(idx) => {
                    if operator != "_"{
                        self.prev_not_concat_op_pos = *idx as i16;
                    }else{
                        self.prev_not_concat_op_pos += 2;
                    }
                    self.result_vec.insert(*idx as usize, operator);
                    self.position_pointer -= 1;
                    true
                }
            };
        } else {
            false
        }
    }

    fn check_conditions(schema: &mut Schema, operator: &String) -> bool {
        let current_position = match schema.positions.get(schema.position_pointer as usize) {
            None => {
                return false;
            }
            Some(position) => *position,
        };

        // let prev_position = match schema.positions.get(schema.position_pointer as usize + 1) {
        //     None => -1,
        //     Some(position) => *position,
        // };

        if operator == "_"
            // && !((schema.prev_op == "" && current_position >= 2) || schema.prev_op == "_")
            && (current_position - schema.prev_not_concat_op_pos as i32) <= 2
        {
            // if !((current_position - prev_position>2) || schema.prev_op == "_"){
            //     println!("DETECTED {:?} {} {}", schema,current_position, prev_position);
            // }else{
            //     println!("{:?} {} {}", schema,current_position, prev_position);
            // }
            // println!("{:?} {} {}", schema,current_position, prev_position);
            false
        } else {
            true
        }
    }

    pub fn _get_positions_len(&self) -> u32 {
        self.positions.len() as u32
    }

    pub fn get_number_of_remain_positions(&self) -> u32 {
        (self.position_pointer + 1) as u32
    }

    pub fn get_number_of_used_positions(&self) -> u32 {
        (self.positions.len() as i16 - self.position_pointer - 1) as u32
    }

    pub fn get_current_position(&self) -> i32 {
        match self.positions.get((self.position_pointer +1) as usize) {
            None => -1,
            Some(position) => *position,
        }
    }
}

pub fn generate_schemes(operands: Vec<String>) -> Vec<Schema> {
    let mut results = Vec::<Schema>::new();
    produce_schema(
        &mut results,
        (operands.len() - 1) as i32,
        (operands.len() - 1) as i32,
        0,
        Vec::<i32>::new(),
    );
    for result in results.iter_mut() {
        result.set_result_vec(operands.clone());
    }
    results
}

fn produce_schema(
    results: &mut Vec<Schema>,
    available: i32,
    max_ops: i32,
    step: i32,
    positions: Vec<i32>,
) {
    if available == 0 && step == (max_ops + 1) {
        results.push(Schema::from_positions(
            positions.into_iter().rev().collect(),
        ));
        return;
    }

    if step < max_ops + 1 {
        produce_schema(results, available, max_ops, step + 1, positions.clone());
    }

    let can_use = min(available, step - 2 - (max_ops - 1 - available));
    if can_use > 0 {
        let mut new_positions = positions;
        new_positions.push(step + (max_ops - available));
        produce_schema(results, available - 1, max_ops, step, new_positions);
    }
}

pub fn _schemes_string(mut schemes: Vec<Schema>, op_sign: &str) -> String {
    let mut answer = String::new();
    for schema in schemes.iter_mut() {
        let ln = schema.positions.len();
        for _ in 0..ln {
            schema.insert_operator(String::from(op_sign));
        }
        answer.push_str(&schema.get_string_result());
        answer.push_str("\n");
    }

    answer
}
