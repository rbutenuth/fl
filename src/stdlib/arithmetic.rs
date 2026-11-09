use crate::{FlError, interpreter::Scope};

enum Operator {
    Plus,
    Minus,
    Times,
    Divide,
    Modulo,
    Exp,
}

impl Operator {
    pub fn execute_i64(&self, left: i64, right: i64) -> Result<i64, FlError> {
        match self {
            Operator::Plus => Ok(left + right),
            Operator::Minus => Ok(left - right),
            Operator::Times => Ok(left * right),
            Operator::Divide => if right == 0 { Err(FlError::new("Division by 0")) } else { Ok(left / right) },
            Operator::Modulo => if right == 0 { Err(FlError::new("Division by 0")) } else { Ok(left % right) },
            Operator::Exp => if left == 0 && right == 0 { Err(FlError::new("0^0")) } else { Ok(left.pow(right as u32)) },
        }
    }

    pub fn execute_f64(&self, left: f64, right: f64) -> Result<f64, FlError> {
        match self {
            Operator::Plus => Ok(left + right),
            Operator::Minus => Ok(left - right),
            Operator::Times => Ok(left * right),
            Operator::Divide => if right == 0.0 { Err(FlError::new("Division by 0")) } else { Ok(left / right) },
            Operator::Modulo => if right == 0.0 { Err(FlError::new("Division by 0")) } else { Ok(left % right) },
            Operator::Exp => if left == 0.0 && right == 0.0 { Err(FlError::new("0^0")) } else { Ok(left.powf(right)) },
        }
    }
}

pub enum Mode {
    Integer,
    FloatingPoint,
}

pub fn add_arithmetic_functions(scope: &dyn Scope) {

/*
        FlFunction::put_in_scope(&*scope, 
        "list",
        "add-front",
        "Return a new list with expression added in front of the given list.",
        vec![
            Parameter::new_from_spec("value", "Value to add", "E").unwrap(),
            Parameter::new_from_spec("list", "List where element is added.", "EL").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::add_at_start(&p[1].as_list()?, p[0].clone())))
        }),
        vec![Sample::new("(add-front 1 '(2))", "(1 2)")]);
*/
}
