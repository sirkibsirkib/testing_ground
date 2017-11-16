mod common_game;
mod noise_test;
mod r_line_test;

use std::collections::HashMap;
extern crate rustyline;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate maplit;

use common_game::Blueprint;
use common_game::BlueprintID;
use common_game::AttributeType;
use common_game::Attributes;
use common_game::Process;
use common_game::BlueprintPool;

extern crate ctrlc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;


fn main() {
    
}
//
// fn f2() {
//     // noise_test::nnn();
// }

// fn f1() {
//     // use AttributeType::*;
//     //
//     // let mut bps = BlueprintPool::new();
//     //
//     // let torch_id = bps.invent(
//     //     Blueprint::new(
//     //
//     //         String::from("torch"),
//     //         Attributes::new(vec!((Mass,3), (Width,4), (Length,4), (Hardness,2))),
//     //     )
//     // );
//     //
//     // let stone_id = bps.invent(
//     //     Blueprint::new(
//     //         String::from("stone"),
//     //         Attributes::new(vec!((Mass,2), (Width,2), (Length,2), (Hardness,9))),
//     //     )
//     // );
//     //
//     // let flattened_torch = bps.apply_process_to_one(torch_id, Process::Flatten);
//     // let flattened_stone = bps.apply_process_to_one(stone_id, Process::Flatten);
//     // println!("{:#?}", bps);
// }
