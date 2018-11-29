extern crate rand;
extern crate reqwest;
#[macro_use]
extern crate json;
extern crate kankyo;

use rand::prelude::*;
use std::collections::HashMap;
use std::io::Read;

const SAMPLE_COUNT: i64 = 10000;

struct Tested (Vec<i32>);

impl ToString for Tested {
    fn to_string(&self) -> String {
        let data = self.0.iter()
            .fold(HashMap::new(), |mut x, acc| {
                let new_num = (x.get(acc).unwrap_or(&0)) + 1;
                x.insert(*acc, new_num);

                x
            });

        let should_roll: i32 = ((1 as f64/data.len() as f64) * (SAMPLE_COUNT as f64)) as i32;

        let mut s = String::new();
        for (i, count) in data {
            let accuracy: i32 = (((should_roll - ((should_roll - count).abs())) as f64/should_roll as f64) * 100 as f64) as i32;
            s.push_str(&format!("Number {} got rolled {} times. Accuracy: %{}.\n", i, count, accuracy));
        }

        s
    }
}

fn test_gen<F: FnMut(i64) -> i32>(mut gen: F) -> Tested {
    let mut samples = Vec::new();

    for i in 0..SAMPLE_COUNT {
        let n = gen(i);
        samples.push(n);
    }

    Tested(samples)
}

fn main() {
    kankyo::load().unwrap();
    let random_org_key = std::env::var("RANDOM_ORG_KEY").unwrap();

    let mut rng = thread_rng();

    let pseudo = test_gen(|_| rng.gen_range(1, 7));
    println!("PSEUDO GENERATION");
    println!("{}", pseudo.to_string());

    let true_client = reqwest::Client::new();
    let body: json::JsonValue = object!{
        "jsonrpc" => "2.0",
        "method" => "generateIntegers",
        "params" => object!{
            "apiKey" => random_org_key,
            "n" => SAMPLE_COUNT,
            "min" => 1,
            "max" => 6
        },
        "id" => 1
    };

    let mut res = true_client.get("https://api.random.org/json-rpc/1/invoke").body(body.to_string()).send().unwrap();
    let mut res_str = String::new();
    res.read_to_string(&mut res_str).unwrap();

    let res_d = json::parse(&res_str).unwrap();
    let data = &res_d["result"]["random"]["data"];

    let true_r = test_gen(|i| data[i as usize].as_i32().unwrap());
    println!("TRUE RANDOM");
    println!("{}", true_r.to_string());
}
