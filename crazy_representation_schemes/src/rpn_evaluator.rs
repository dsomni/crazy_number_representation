use std::collections::hash_map::HashMap;

const LOWER_BOUND: f64 = 0.001;
const UPPER_BOUND: f64 = 1073741824.0;

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

fn tokenizer(expr: &str) -> Result<Vec<OperationElt>, String> {
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

pub fn evaluate(expr: &str) -> Result<f64, String> {
    return match tokenizer(expr) {
        Ok(tokens) => {
            let mut stack: Vec<f64> = Vec::new();
            for token in tokens {
                match token {
                    OperationElt::Operator(operator) => {
                        if stack.len() < 2 {
                            return Err("Unsufficient operands before operator".to_string());
                        }

                        let operand2 = match stack.pop() {
                            None => {
                                return Err("expected f64 values in stack".to_string());
                            }
                            Some(x) => x,
                        };

                        let operand1 = match stack.pop() {
                            None => {
                                return Err("expected f64 values in stack".to_string());
                            }
                            Some(x) => x,
                        };
                        let result = match operator {
                            Operator::Addition => operand1 + operand2,
                            Operator::Substraction => operand1 - operand2,
                            Operator::Multiplication => operand1 * operand2,
                            Operator::Division => {
                                let op_result = operand1 / operand2;
                                if op_result < LOWER_BOUND || op_result > UPPER_BOUND {
                                    return Err("Number is too big or too small".to_string());
                                } else {
                                    op_result
                                }
                            }
                            Operator::Concatenation => {
                                // let s_o = operand2.to_string();
                                // operand1 * 10.0_f64.powi(s_o.len() as i32) + operand2
                                match format!("{}{}", operand1, operand2).parse::<f64>() {
                                    Ok(x) => x,
                                    Err(_) => {
                                        return Err("Concatenation is unreal".to_string());
                                    }
                                }
                            }
                            Operator::Power => {
                                let op_result = operand1.powf(operand2);
                                if op_result < LOWER_BOUND || op_result >= UPPER_BOUND {
                                    return Err("Number is too big or too small".to_string());
                                } else {
                                    op_result
                                }
                            }
                        };
                        stack.push(result);
                    }
                    OperationElt::Operand(val) => stack.push(val),
                }
            }
            if stack.len() != 1 {
                return Err("Remaining untreated operands. Probably missing operator.".to_string());
            }

            return match stack.pop() {
                None => Err("expected f64 values in stack".to_string()),
                Some(x) => Ok(x),
            };
        }
        Err(err) => Err(err),
    };
}

fn _calculate(result: &str) -> f64 {
    evaluate(result).unwrap()
}

struct _Entity {
    value: String,
    last_op_priority: u8,
}

impl _Entity {
    fn _new(value: f64) -> _Entity {
        _Entity {
            value: (value as i64).to_string(),
            last_op_priority: 127,
        }
    }

    fn _from(value: String, last_op_priority: u8) -> _Entity {
        _Entity {
            value,
            last_op_priority,
        }
    }
}

pub fn to_infix(expr: &str) -> Result<String, String> {
    let mut priority_map = HashMap::<String, u8>::new();
    priority_map.insert(String::from("+"), 1);
    priority_map.insert(String::from("-"), 1);
    priority_map.insert(String::from("*"), 2);
    priority_map.insert(String::from("/"), 2);
    priority_map.insert(String::from("^"), 3);
    priority_map.insert(String::from("_"), 4);

    match evaluate(expr) {
        Ok(_) => {
            let tokens = tokenizer(expr).unwrap();
            let mut stack: Vec<_Entity> = Vec::new();
            for token in tokens {
                match token {
                    OperationElt::Operator(operator) => {
                        if stack.len() < 2 {
                            return Err("Unsufficient operands before operator".to_string());
                        }

                        let operand2 = match stack.pop() {
                            None => {
                                return Err("expected f64 values in stack".to_string());
                            }
                            Some(x) => x,
                        };
                        let operand2_priority = operand2.last_op_priority;
                        let operand2_value = operand2.value;

                        let operand1 = match stack.pop() {
                            None => {
                                return Err("expected f64 values in stack".to_string());
                            }
                            Some(x) => x,
                        };
                        let operand1_priority = operand1.last_op_priority;
                        let operand1_value = operand1.value;

                        let result_priority: u8;
                        let result = match operator {
                            Operator::Addition => {
                                result_priority = *priority_map.get("+").unwrap();
                                format!("{} + {}", operand1_value, operand2_value)
                            }
                            Operator::Substraction => {
                                result_priority = *priority_map.get("-").unwrap();
                                if operand2_priority <= result_priority {
                                    format!("{} - ({})", operand1_value, operand2_value)
                                } else {
                                    format!("{} - {}", operand1_value, operand2_value)
                                }
                            }
                            Operator::Multiplication => {
                                result_priority = *priority_map.get("*").unwrap();
                                let current_op_priority = result_priority;
                                if operand1_priority < current_op_priority
                                    && !(operand2_priority < current_op_priority)
                                {
                                    format!("({}) * {}", operand1_value, operand2_value)
                                } else if !(operand1_priority < current_op_priority)
                                    && operand2_priority < current_op_priority
                                {
                                    format!("{} * ({})", operand1_value, operand2_value)
                                } else if operand1_priority < current_op_priority
                                    && operand2_priority < current_op_priority
                                {
                                    format!("({}) * ({})", operand1_value, operand2_value)
                                } else {
                                    format!("{} * {}", operand1_value, operand2_value)
                                }
                            }
                            Operator::Division => {
                                result_priority = *priority_map.get("/").unwrap();
                                let current_op_priority = result_priority;
                                if operand1_priority <= current_op_priority
                                    && !(operand2_priority <= current_op_priority)
                                {
                                    format!("({}) / {}", operand1_value, operand2_value)
                                } else if !(operand1_priority <= current_op_priority)
                                    && operand2_priority <= current_op_priority
                                {
                                    format!("{} / ({})", operand1_value, operand2_value)
                                } else if operand1_priority <= current_op_priority
                                    && operand2_priority <= current_op_priority
                                {
                                    format!("({}) / ({})", operand1_value, operand2_value)
                                } else {
                                    format!("{} / {}", operand1_value, operand2_value)
                                }
                            }
                            Operator::Concatenation => {
                                result_priority = *priority_map.get("_").unwrap();
                                let current_op_priority = result_priority;
                                if operand1_priority < current_op_priority
                                    && !(operand2_priority < current_op_priority)
                                {
                                    format!("({}){}", operand1_value, operand2_value)
                                } else if !(operand1_priority < current_op_priority)
                                    && operand2_priority < current_op_priority
                                {
                                    format!("{}({})", operand1_value, operand2_value)
                                } else if operand1_priority < current_op_priority
                                    && operand2_priority < current_op_priority
                                {
                                    format!("({})({})", operand1_value, operand2_value)
                                } else {
                                    format!("{}{}", operand1_value, operand2_value)
                                }
                            }
                            Operator::Power => {
                                result_priority = *priority_map.get("^").unwrap();
                                let current_op_priority = result_priority;
                                if operand1_priority < current_op_priority
                                    && !(operand2_priority < current_op_priority)
                                {
                                    format!("({}) ^ {}", operand1_value, operand2_value)
                                } else if !(operand1_priority < current_op_priority)
                                    && operand2_priority < current_op_priority
                                {
                                    format!("{} ^ ({})", operand1_value, operand2_value)
                                } else if operand1_priority < current_op_priority
                                    && operand2_priority < current_op_priority
                                {
                                    format!("({}) ^ ({})", operand1_value, operand2_value)
                                } else {
                                    format!("{} ^ {}", operand1_value, operand2_value)
                                }
                            }
                        };
                        stack.push(_Entity::_from(result, result_priority));
                    }
                    OperationElt::Operand(val) => stack.push(_Entity::_new(val)),
                }
            }

            if stack.len() != 1 {
                return Err("Remaining untreated operands. Probably missing operator.".to_string());
            }

            return match stack.pop() {
                None => Err("expected f64 values in stack".to_string()),
                Some(x) => Ok(x.value),
            };
        }
        Err(_) => Err(String::from("Could not evaluate the expression")),
    }
}
