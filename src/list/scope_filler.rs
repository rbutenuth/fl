use crate::{Value, interpreter::{FlFunction, Scope}, list::FlList};
use crate::interpreter::function::Parameter;

pub fn add_list_functions(scope: &dyn Scope) {

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
        }));

        FlFunction::put_in_scope(&*scope, 
        "list",
        "add-end",
        "Return a new list with expression added in front of the given list.",
        vec![
            Parameter::new_from_spec("list", "List where element is added.", "EL").unwrap(),
            Parameter::new_from_spec("value", "Value to add.", "E").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::add_at_end(&p[0].as_list()?, p[1].clone())))
        }));

        FlFunction::put_in_scope(&*scope, 
        "list",
        "append",
        "Return the concatenation of two lists.",
        vec![
            Parameter::new_from_spec("first", "First list.", "EL").unwrap(),
            Parameter::new_from_spec("second", "Second list", "EL").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::append(&p[0].as_list()?, &p[1].as_list()?)))
        }));

/* TODO:

pub fn first(&self) -> Result<Value, FlError> {
pub fn last(&self) -> Result<Value, FlError> {
pub fn get(&self, index: isize) -> Result<Value, FlError> {
pub fn set(&self, position: usize, element: Value) -> Result<FlList, FlError> {
pub fn replace_elements(

is-list
list

pub fn lower_half(&self) -> FlList {
pub fn upper_half(&self) -> FlList {

pub fn remove_first(&self) -> Result<FlList, FlError> {
pub fn remove_last(&self) -> Result<FlList, FlError> {

*/
        FlFunction::put_in_scope(&*scope, 
        "list",
        "size",
        "Number of elements.",
        vec![
            Parameter::new_from_spec("list", "list to check", "EL").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::Integer(FlList::len(&p[0].as_list()?) as i64))
        }));

        FlFunction::put_in_scope(&*scope, 
        "list",
        "is-empty",
        "Is this list, string or object empty (size 0)? Nil is empty, too",
        vec![
            Parameter::new_from_spec("value", "value to check", "E").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::Integer(
            match &p[0] {
                Value::Nil => 1,
                Value::Integer(i) => if *i == 0 { 1 } else { 0 },
                Value::Float(f) => if *f == 0.0 { 1 } else { 0 },
                Value::List(list) => if list.is_empty() { 1 } else { 0 },
                Value::Symbol(_symbol) => 0,
                Value::Text(s) => if s.len() == 0 { 1 } else { 0 },
                Value::Map() => todo!(),
                Value::Object() => todo!(),
                Value::Function(_fl_function) => 0,
            }))
        }));

/* TODO:

pub fn sub_list(&self, from_index: usize, to_index: usize) -> Result<FlList, FlError> {

sort (with comparator)

 */
}
