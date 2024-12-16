#[macro_use]
extern crate rulinalg;
use clap::Parser;
use rayon::prelude::*;
use std::fmt;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

use ss::Set;
use ss::SetSystem;

mod algos;
mod ss;

/// Compute a low-crossing partition of a set system
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the algorithm used to create the partition
    #[arg(short, long)]
    algo: Algo,

    /// Number of points in the set system
    #[arg(short, long)]
    number: Option<i32>,

    /// Number of parts in the partition
    #[arg(short, long)]
    tpart: i32,

    /// Dimension of the set system
    #[arg(short, long)]
    dimension: usize,

    /// Type of set system to generate
    #[arg(short, long)]
    setsystem: String,

    /// Name of file to save the result
    #[arg(short, long)]
    output: Option<String>,

    /// Name of file to save the set system generated
    #[arg(short, long)]
    generate: Option<String>,

    /// Write the result stats to a file
    #[arg(short, long)]
    write: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Algo {
    Min,
    AO,
    Potential,
}

impl fmt::Display for Algo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Algo::Min => write!(f, "min"),
            Algo::AO => write!(f, "atonce"),
            Algo::Potential => write!(f, "potential"),
        }
    }
}

fn main() {
    let args = Args::parse();
    let d: usize = args.dimension;
    let t = args.tpart;
    let n: i32;
    let ss: SetSystem;
    match args.setsystem.as_str() {
        "grid" => {
            n = args.number.expect("n is required for the grid set system");
            ss = SetSystem::grid(n, d);
        }
        "rhs" => {
            n = args.number.expect("n is required for the grid set system");
            ss = SetSystem::rhs(n, ((n as f32) * (n as f32).ln()).floor() as i32, d);
        }
        x => {
            ss = SetSystem::from_file(x);
            n = ss.points.len() as i32;
        }
    }
    match args.output {
        None => (),
        Some(x) => ss.to_file(x.as_str()),
    }
    let res: SetSystem;
    let time: Duration;
    match args.algo {
        Algo::Min => {
            (res, time) = algos::part_min(&ss, t);
        }
        Algo::AO => {
            (res, time) = algos::part_at_once(&ss, t, (n as f32).sqrt() as i32);
        }
        Algo::Potential => {
            (res, time) = algos::part_potential(&ss, t);
        }
    }
    let intersections = intersections(&res.sets, &ss.sets);
    println!(
        "Intersections : max -> {}, avg -> {}, min -> {}",
        intersections
            .iter()
            .max()
            .expect("Fail to determine maximum"),
        mean(&intersections),
        intersections
            .iter()
            .min()
            .expect("Fail to determine intersection min")
    );
    match args.write {
        None => (),
        Some(x) => {
            let mut file = OpenOptions::new().write(true).append(true).open(x).unwrap();

            if let Err(e) = writeln!(
                file,
                "{};{};{};{};{};{};{};{};{};{};{};{:.4};{};{}",
                args.algo,
                n,
                t,
                args.setsystem,
                ss.sets.len(),
                d,
                0,
                intersections.iter().max().expect(""),
                mean(&intersections),
                intersections.iter().min().expect(""),
                0,
                time.as_secs_f64(),
                0,
                0
            ) {
                eprintln!("Couldn't write to file: {}", e);
            }
        }
    }
    match args.generate {
        None => (),
        Some(x) => res.to_file(x.as_str()),
    }
}

fn mean(v: &Vec<i32>) -> f32 {
    let mut sum = 0;
    for x in v.iter() {
        sum += x;
    }
    sum as f32 / v.len() as f32
}

fn intersections(parts: &Vec<Set>, ss: &Vec<Set>) -> Vec<i32> {
    let n = parts[0].points.len();
    let inter = ss.par_iter().map(|s| intersection(parts, s, n)).collect();
    inter
}

fn intersection(parts: &Vec<Set>, s: &Set, n: usize) -> i32 {
    let mut res = 0;
    for p in parts.iter() {
        let mut start: usize = n + 1;
        for i in 0..p.points.len() {
            if p.points[i] {
                if start == n + 1 {
                    start = i;
                } else if algos::intersects((start, i), s) {
                    res += 1;
                    break;
                }
            }
        }
    }
    res
}
