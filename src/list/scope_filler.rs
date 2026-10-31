use crate::{Value, interpreter::{FlFunction, Scope}, list::FlList};
use crate::interpreter::function::Parameter;

pub fn add_list_functions(scope: &dyn Scope) {

/* TODO:

pub fn add_at_start(&self, value: Value) -> FlList {
pub fn add_at_end(&self, value: Value) -> FlList {
pub fn append(&self, list: &FlList) -> FlList {
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

pub fn len(&self) -> usize {

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

/*

pub fn sub_list(&self, from_index: usize, to_index: usize) -> Result<FlList, FlError> {

sort (with comparator)

 */
}
