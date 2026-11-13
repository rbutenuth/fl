use std::sync::Arc;
use crate::{FlError, Value, interpreter::{FlFunction, Scope}};
use crate::interpreter::function::{Sample, Parameter};

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
            Operator::Divide => if right == 0 { Err(FlError::new("division by 0")) } else { Ok(left / right) },
            Operator::Modulo => if right == 0 { Err(FlError::new("division by 0")) } else { Ok(left % right) },
            Operator::Exp => if left == 0 && right == 0 { Err(FlError::new("0^0")) } else { Ok(left.pow(right as u32)) },
        }
    }

    pub fn execute_f64(&self, left: f64, right: f64) -> Result<f64, FlError> {
        match self {
            Operator::Plus => Ok(left + right),
            Operator::Minus => Ok(left - right),
            Operator::Times => Ok(left * right),
            Operator::Divide => if right == 0.0 { Err(FlError::new("division by 0")) } else { Ok(left / right) },
            Operator::Modulo => if right == 0.0 { Err(FlError::new("division by 0")) } else { Ok(left % right) },
            Operator::Exp => if left == 0.0 && right == 0.0 { Err(FlError::new("0^0")) } else { Ok(left.powf(right)) },
        }
    }
}

impl Operator {
    fn symbol(&self) -> &'static str {
        match self {
            Operator::Plus => "+",
            Operator::Minus => "-",
            Operator::Times => "*",
            Operator::Divide => "/",
            Operator::Modulo => "%",
            Operator::Exp => "**",
        }
    }
}

/// A number is either integer or floating point; computation starts in integer
/// mode and switches to floating point as soon as a float is seen.
enum Accumulator {
    Integer(i64),
    Float(f64),
}

fn to_accumulator(operator: &Operator, value: &Value) -> Result<Accumulator, FlError> {
    match value {
        Value::Integer(i) => Ok(Accumulator::Integer(*i)),
        Value::Float(f) => Ok(Accumulator::Float(*f)),
        other => Err(FlError::new(format!(
            "{} does not work on {:?}",
            operator.symbol(),
            other.as_value_type()
        ))),
    }
}

/// Apply `operator` from left to right: `(- 10 2 3)` is `(10 - 2) - 3`.
/// A single value is returned as it is, except for unary minus, which negates it.
/// As soon as one value is a float, the rest of the computation is done with floats.
/// The values are already evaluated.
fn compute(operator: Operator, values: &Vec<Value>) -> Result<Value, FlError> {
    let Some((first, rest)) = values.split_first() else {
        return Err(FlError::new(format!("{} needs at least one parameter", operator.symbol())));
    };
    let mut acc = to_accumulator(&operator, first)?;

    if matches!(operator, Operator::Minus) && rest.is_empty() {
        return Ok(match acc {
            Accumulator::Integer(i) => Value::Integer(-i),
            Accumulator::Float(f) => Value::Float(-f),
        });
    }

    for value in rest {
        acc = match (acc, to_accumulator(&operator, value)?) {
            (Accumulator::Integer(l), Accumulator::Integer(r)) =>
                Accumulator::Integer(operator.execute_i64(l, r)?),
            (Accumulator::Integer(l), Accumulator::Float(r)) =>
                Accumulator::Float(operator.execute_f64(l as f64, r)?),
            (Accumulator::Float(l), Accumulator::Integer(r)) =>
                Accumulator::Float(operator.execute_f64(l, r as f64)?),
            (Accumulator::Float(l), Accumulator::Float(r)) =>
                Accumulator::Float(operator.execute_f64(l, r)?),
        };
    }

    Ok(match acc {
        Accumulator::Integer(i) => Value::Integer(i),
        Accumulator::Float(f) => Value::Float(f),
    })
}

pub fn add_arithmetic_functions(scope: &dyn Scope) {

        FlFunction::put_in_scope(&*scope, 
        "arithmetic",
        Operator::Plus.symbol(),
        "Add integer or floating point numbers.",
        vec![
            Parameter::new_from_spec("value", "Values to add", "EID").unwrap(),
        ],
        true,
        Box::new(|_scope: &Arc<dyn Scope>, p: &Vec<Value>|{
          compute(Operator::Plus, p)
        }),
        vec![Sample::new("(+ 1 2)", "3")]);

        FlFunction::put_in_scope(&*scope, 
        "arithmetic",
        Operator::Minus.symbol(),
        "Unary minus (one parameter) or subtract values from first one.",
        vec![
            Parameter::new_from_spec("value", "Values to subtract", "EID").unwrap(),
        ],
        true,
        Box::new(|_scope: &Arc<dyn Scope>, p: &Vec<Value>|{
            compute(Operator::Minus, p)
        }),
        vec![Sample::new("(- 3 2)", "1"), Sample::new("(- 3)", "-3")]);

        FlFunction::put_in_scope(&*scope, 
        "arithmetic",
        Operator::Times.symbol(),
        "Multiply integer or floating point numbers.",
        vec![
            Parameter::new_from_spec("value", "Values to multiply", "EID").unwrap(),
        ],
        true,
        Box::new(|_scope: &Arc<dyn Scope>, p: &Vec<Value>|{
            compute(Operator::Times, p)
        }),
        vec![Sample::new("(* 3 2)", "6")]);

        FlFunction::put_in_scope(&*scope, 
        "arithmetic",
        Operator::Divide.symbol(),
        "Divide integer or floating point numbers.",
        vec![
            Parameter::new_from_spec("value", "Values to divide", "EID").unwrap(),
        ],
        true,
        Box::new(|_scope: &Arc<dyn Scope>, p: &Vec<Value>|{
            compute(Operator::Divide, p)
        }),
        vec![
            Sample::new("(/ 8 2)", "4"), 
            Sample::new("(/ 3 0)", "Error: division by 0"),
            Sample::new("(/ 3.0 2)", "1.5")]);

        FlFunction::put_in_scope(&*scope, 
        "arithmetic",
        Operator::Modulo.symbol(),
        "Modulo of integer or floating point numbers.",
        vec![
            Parameter::new_from_spec("value", "Values to divide", "EID").unwrap(),
        ],
        true,
        Box::new(|_scope: &Arc<dyn Scope>, p: &Vec<Value>|{
            compute(Operator::Modulo, p)
        }),
        vec![
                Sample::new("(% 9 2)", "1"),
                Sample::new("(% 3 0)", "Error: division by 0"),
                Sample::new("(% 4 2)", "0")]);

        FlFunction::put_in_scope(&*scope, 
        "arithmetic",
        Operator::Exp.symbol(),
        "a^b (exponentiation) of integer or floating point numbers.",
        vec![
            Parameter::new_from_spec("value", "Values to exponentiate", "EID").unwrap(),
        ],
        true,
        Box::new(|_scope: &Arc<dyn Scope>, p: &Vec<Value>|{
            compute(Operator::Exp, p)
        }),
        vec![Sample::new("(** 2 10)", "1024"), Sample::new("(** 0 0)", "Error: 0^0")]);
}
