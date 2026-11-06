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
        }),
        Vec::new());

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
        }),
        Vec::new());

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
        }),
        Vec::new());

        FlFunction::put_in_scope(&*scope, 
        "list",
        "first",
        "First element, error on empty list.",
        vec![
            Parameter::new_from_spec("list", "Non empty list.", "EL").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            FlList::first(&p[0].as_list()?)
        }),
        Vec::new());

        FlFunction::put_in_scope(&*scope, 
        "list",
        "last",
        "Last element, error on empty list.",
        vec![
            Parameter::new_from_spec("list", "Non empty list.", "EL").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            FlList::last(&p[0].as_list()?)
        }),
        Vec::new());
        
        FlFunction::put_in_scope(&*scope, 
        "list",
        "get-element",
        "Get one element from a non empty list.",
        vec![
            Parameter::new_from_spec("list", "Non empty list.", "EL").unwrap(),
            Parameter::new_from_spec("index", "Index (starting at 0) of element to get.", "EI").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            FlList::get(&p[0].as_list()?, p[1].as_integer()? as isize)
        }),
        Vec::new());
        
        FlFunction::put_in_scope(&*scope, 
        "list",
        "set-element",
        "Set (replace) one element of a list.",
        vec![
            Parameter::new_from_spec("list", "Non empty list.", "EL").unwrap(),
            Parameter::new_from_spec("index", "Index (starting at 0) of element to set.", "EI").unwrap(),
            Parameter::new_from_spec("element", "New value.", "E").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::set(&p[0].as_list()?, p[1].as_integer()? as isize, p[2].clone())?))
        }),
        Vec::new());
        
        FlFunction::put_in_scope(&*scope, 
        "list",
        "replace-elements",
        "Replace some elements from this list with elements of another list. \
              It is not required for the second list to have the same number of elements \
              as elements are removed from the original list. from is the index of the \
              first replaced element, new-elements is the list with the new elements, \
              num-replaced elements will be removed.",
        vec![
            Parameter::new_from_spec("list", "list.", "EL").unwrap(),
            Parameter::new_from_spec("from", "Index of first replaced element (starting at 0)", "EI").unwrap(),
            Parameter::new_from_spec("new-elements", "A list with new elements.", "EL").unwrap(),
            Parameter::new_from_spec("num-replaced", "Number of elements to remove which will be removed.", "EI").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::replace_elements(
                &p[0].as_list()?, 
                p[1].as_integer()? as isize, 
                &p[2].as_list().clone()?,
                p[3].as_integer()? as isize
            )?))
        }),
        Vec::new());

        FlFunction::put_in_scope(&*scope, 
        "list",
        "is-list",
        "Returns 1 when this is a list, 0 otherwise.",
        vec![
            Parameter::new_from_spec("value", "Value to test.", "E").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::Integer(match p[0] {
                Value::List(_) => 1,
                _ => 0,
            }))
        }),
        Vec::new());

        FlFunction::put_in_scope(&*scope, 
        "list",
        "list",
        "Create a list from the parameters.",
        vec![
            Parameter::new_from_spec("values", "The values to create a list from.", "E").unwrap(),
        ],
        true,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::from_iterator_values(p.iter().cloned())))
        }),
        Vec::new());

        FlFunction::put_in_scope(&*scope, 
        "list",
        "lower-half",
        "Return the lower half of a list (opposite to upper-half). \
                 In case the number of elements is not even, the lower half \
                 has one element less than the upper half.",
        vec![
            Parameter::new_from_spec("list", "The list to split", "EL").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::lower_half(&p[0].as_list()?)))
        }),
        Vec::new());

        FlFunction::put_in_scope(&*scope, 
        "list",
        "upper-half",
        "Return the upper half of a list (opposite to lower-half). \
                 In case the number of elements is not even, the lower half \
                 has one element less than the upper half.",
        vec![
            Parameter::new_from_spec("list", "The list to split", "EL").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::upper_half(&p[0].as_list()?)))
        }),
        Vec::new());


        FlFunction::put_in_scope(&*scope, 
        "list",
        "remove-last",
        "Remove last element of a list.",
        vec![
            Parameter::new_from_spec("list", "Non empty list.", "EL").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::remove_last(&p[0].as_list()?)?))
        }),
        Vec::new());

        FlFunction::put_in_scope(&*scope, 
        "list",
        "remove-first",
        "Remove first element of a list.",
        vec![
            Parameter::new_from_spec("list", "Non empty list.", "EL").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::remove_first(&p[0].as_list()?)?))
        }),
        Vec::new());

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
        }),
        Vec::new());

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
        }),
        Vec::new());

        FlFunction::put_in_scope(&*scope, 
        "list",
        "sub-list",
        "Return a part from the given list, including start, excluding end (counted from 0).",
        vec![
            Parameter::new_from_spec("list", "list.", "EL").unwrap(),
            Parameter::new_from_spec("start", "Start index of elements taken (starting at 0)", "EI").unwrap(),
            Parameter::new_from_spec("end", "Index of first element not in the result list.", "EI").unwrap(),
        ],
        false,
        Box::new(|_scope: &dyn Scope, p: &Vec<Value>|{
            Ok(Value::List(FlList::sub_list(
                &p[0].as_list()?, 
                p[1].as_integer()? as isize, 
                p[2].as_integer()? as isize)?))
        }),
        Vec::new());

/* TODO:

sort (with comparator)

 */
}

/*

#[cfg(test)]
mod tests {
    use std::sync::LazyLock;
    use super::*;

    static SHARED: LazyLock<ExpensiveThing> = LazyLock::new(|| ExpensiveThing::new());

    #[test]
    fn test_a() {
        let x = &*SHARED;   // first test to get here builds it, the others reuse it
        // ...
    }

    #[test]
    fn test_b() {
        assert!(SHARED.something());
    }
}
*/