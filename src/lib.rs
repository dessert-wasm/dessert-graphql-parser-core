// Needed by pest
#![recursion_limit = "300"]

mod utils;
use wasm_bindgen::prelude::*;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use pest::iterators::Pairs;

use serde_json::{json, value::Value};
use std::str::FromStr;

extern crate web_sys;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[derive(Parser)]
#[grammar = "graph.pest"]
struct GraphParser;

fn merge(a: &mut Value, b: &Value) {
    match (a, b) {
        (&mut Value::Object(ref mut a), &Value::Object(ref b)) => {
            for (k, v) in b {
                merge(a.entry(k.clone()).or_insert(Value::Null), v);
            }
        }
        (a, b) => {
            *a = b.clone();
        }
    }
}

pub fn get_inner_size(pairs: Pairs<Rule>) -> u32 {
    let mut i = 0;
    for pair in pairs {
        i += 1;
    }
    i
}

pub fn parse_object(pairs: Pairs<Rule>) -> Value {
    let mut value = json!({});
    for pair in pairs {
        match pair.as_rule() {
            Rule::arg => {
                merge(&mut value, &parse_arg(pair.into_inner()));
            }
            _ => ()
        }
    }
    value
}

pub fn parse_value(pairs: Pairs<Rule>) -> Value {
    let mut value = json!({});

    for pair in pairs {
        let str = pair.as_str();

        match pair.as_rule() {
            Rule::variable => {
                value = json!(pair.as_str());
            }
            Rule::float => {
                value = json!(f32::from_str(str).unwrap());
            }
            Rule::int => {
                value = json!(i32::from_str(str).unwrap());
            }
            Rule::string => { //String is like "\"...\""
                let mut unescape = String::from(str);
                unescape.remove(0);
                unescape.remove(unescape.len() - 1);
                value = json!(unescape);
            }
            Rule::boolean => {
                value = json!(bool::from_str(str).unwrap());
            }
            Rule::null => {
                value = json!(null);
            }
            Rule::enum_val => {
                //TODO
            }
            Rule::list => {
                let mut fields: Vec<Value> = Vec::default();
                for inner_pair in pair.into_inner() {
                    fields.push(parse_value(inner_pair.into_inner()));
                }
                value = json!(fields);
            }
            Rule::object => {
                value = parse_object(pair.into_inner());
            }
            _ => ()
        }
    }
    value
}

pub fn parse_arg(pairs: Pairs<Rule>) -> Value {
    let mut name = String::default();
    let mut value = json!({});

    for pair in pairs {
        match pair.as_rule() {
            Rule::name => {
                name = String::from(pair.as_str());
            }
            Rule::value => {
                value = parse_value(pair.into_inner());
            }
            _ => ()
        }
    }
    json!({name: value})
}

pub fn parse_args(pairs: Pairs<Rule>) -> Value {
    let mut args: Vec<Value> = Vec::default();

    for pair in pairs {
        match pair.as_rule() {
            Rule::arg => {
                let arg = parse_arg(pair.into_inner());
                args.push(arg);
            }
            _ => ()
        }
    }
    json!(args)
}

pub fn parse_field(pairs: Pairs<Rule>) -> Value {
    let mut value = json!({});

    for pair in pairs {
        match pair.as_rule() {
            Rule::name => {
                value["name"] = json!(pair.as_str());
            }
            Rule::field => {
                value["field"] = parse_field(pair.into_inner());
            }
            Rule::selection_set => {
                value["selection"] = parse_selection(pair.into_inner());
            }
            Rule::args => {
                value["args"] = parse_args(pair.into_inner());
            }
            Rule::alias => {
                value["alias"] = json!(pair.as_str());
            }
            _ => ()
        }
    }
    value
}

pub fn parse_directive(pairs: Pairs<Rule>) -> Value {
    let mut value = json!({});

    for pair in pairs {
        match pair.as_rule() {
            Rule::name => {
                value["name"] = json!(pair.as_str());
            }
            Rule::args => {
                value["args"] = parse_args(pair.into_inner());
            }
            _ => ()
        }
    }
    value
}

pub fn parse_fragment_inline(pairs: Pairs<Rule>) -> Value {
    let mut value = json!({});

    for pair in pairs {
        match pair.as_rule() {
            Rule::name => {
                value["name"] = json!(pair.as_str());
            }
            Rule::selection_set => {
                value["selection"] = parse_selection(pair.into_inner());
            }
            Rule::directive => {
                value["directive"] = parse_directive(pair.into_inner());
            }
            _ => ()
        }
    }
    value
}

pub fn parse_selection(pairs: Pairs<Rule>) -> Value {
    let mut value = json!({});
    let mut fields: Vec<Value> = Vec::default();

    for pair in pairs {
        match pair.as_rule() {
            Rule::field => {
                let field = parse_field(pair.into_inner());
                fields.push(field);
            }
            Rule::fragment_spread => {
                let fragment = json!({"spread fragment": pair.as_str()});
                fields.push(fragment);
            }
            Rule::fragment_inline => {
                let fragment = parse_fragment_inline(pair.into_inner());
                fields.push(json!({"inline fragment":fragment}));
            }
            Rule::selection_set => {
                value["selection"] = parse_selection(pair.into_inner());
            }
            _ => ()
        }
    }

    if !fields.is_empty() {
        value["fields"] = json!(fields);
    }
    value
}

pub fn parse_variable_def(pairs: Pairs<Rule>) -> Value {
    let mut value = json!({});

    for pair in pairs {
        match pair.as_rule() {
            Rule::types => {
                value["type"] = json!(pair.as_str());
            }
            Rule::variable => {
                value["name"] = json!(pair.as_str());
            }
            _ => ()
        }
    }
    value
}

pub fn parse_operation(pairs: Pairs<Rule>) -> Value {
    let mut value = json!({});
    let mut variables: Vec<Value> = Vec::default();

    for pair in pairs {
        match pair.as_rule() {
            Rule::name => {
                value["name"] = json!(pair.as_str());
            }
            Rule::variable_def => {
                let def = parse_variable_def(pair.into_inner());
                variables.push(def);
            }
            Rule::field => {
                value["field"] = parse_field(pair.into_inner());
            }
            Rule::selection_set => {
                value["selection"] = parse_selection(pair.into_inner());
            }

            _ => ()
        }
    }

    if !variables.is_empty() {
        value["variables"] = json!(variables);
    }
    value
}

pub fn parse_values(pairs: Pairs<Rule>) -> Value {
    let mut value = json!({});
    let mut operations: Vec<Value> = Vec::default();
    let size = get_inner_size(pairs.clone()) - 1; //-1 remove EOI

    for pair in pairs {
        match pair.as_rule() {
            Rule::query => {
                let op = parse_operation(pair.into_inner());
                if size > 1 {
                    operations.push(json!({"query": op}));
                } else {
                    value["query"] = op;
                }
            }
            Rule::mutation => {
                let op = parse_operation(pair.into_inner());
                if size > 1 {
                    operations.push(json!({"mutation": op}));
                } else {
                    value["mutation"] = op;
                }
            }
            Rule::selection_set => {
                let op = parse_selection(pair.into_inner());
                if size > 1 {
                    operations.push(json!({"selection": op}));
                } else {
                    value["selection"] = op;
                }
            }
            Rule::fragment_def => {
                let op = parse_operation(pair.into_inner());
                if size > 1 {
                    operations.push(json!({"fragment": op}));
                } else {
                    value["fragment"] = op;
                }
            }
            _ => ()
        }
    }

    if size > 1 {
        value = json!(operations);
    }
    value
}

#[wasm_bindgen]
pub fn parse_graphql(document: &str) -> JsValue {
    return match GraphParser::parse(Rule::document, document) {
        Ok(pairs) => {
            JsValue::from_serde(&parse_values(pairs)).unwrap()
        }
        Err(e) => {
            if document.is_empty() {
                return JsValue::from("Given empty string");
            }
            log!("{}", e);
            JsValue::from("unsuccessful parse")
        }
    };
}