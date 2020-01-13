// Needed by pest
#![recursion_limit = "300"]

mod utils;

use wasm_bindgen::prelude::*;

extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use pest::iterators::Pair;
use pest::iterators::Pairs;

use serde;
use serde_json::{json, to_vec_pretty};

extern crate web_sys;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[derive(Parser)]
#[grammar = "graph.pest"]
struct GraphParser;

pub fn parse_field(pairs: Pairs<Rule>) -> serde_json::value::Value {
    let mut fields = Vec::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::name => {
                fields.push(json!({"name" : pair.as_str()}));
            }
            Rule::field => {
                let len = fields.len();
                let tmp = fields.get_mut(len - 1).unwrap();
                tmp["fields"] = parse_field(pair.clone().into_inner());
            }
            _ => ()
        }
    }
    return json!(fields);
}

pub fn parse_value(pairs: Pairs<Rule>, json: &mut serde_json::value::Value) -> JsValue {
    for pair in pairs {
        match pair.as_rule() {
            Rule::operation => { parse_value(pair.clone().into_inner(), json); }
            Rule::query => json["operation"] = json!("query"),
            Rule::mutation => json["operation"] = json!("mutation"),
            _ => ()
        }

        for inner_pair in pair.into_inner() {
            match inner_pair.as_rule() {
                Rule::name => json["name"] = json!(inner_pair.as_str()),
                /*Rule::selection_set => log!("Selection : {}", inner_pair.as_str()),*/
                Rule::field => {
                    json["fields"] = parse_field(inner_pair.clone().into_inner());
                }
                _ => ()
            };
        }
    }
    JsValue::from("Error")
}

pub fn parse_values(pairs: Pairs<Rule>) -> JsValue {
    let mut john = json!({});
    parse_value(pairs, &mut john);
    return JsValue::from_serde(&john).unwrap();
}

#[wasm_bindgen]
pub fn parseDocument(document: &str) -> JsValue {
    return match GraphParser::parse(Rule::document, document) {
        Ok(pairs) => parse_values(pairs),
        Err(e) => {
            log!("{}", e);
            JsValue::from("unsuccessful parse")
        }
    };
}