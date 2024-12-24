#[macro_use]
extern crate rulinalg;
use clap::{Args, Parser, Subcommand};
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
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a set system
    Generate(GenerateArgs),
    /// Partition a set system
    Partition(PartitionArgs),
    /// Computes the intersection of a partition with a set system
    Intersections(IntersectionsArgs),
}

#[derive(Args)]
struct GenerateArgs {
    /// Type of set system to generate (grid or rhs)
    #[arg(short, long)]
    sstype: String,

    /// Dimension of the set system
    #[arg(short, long)]
    dimension: usize,

    /// Number of points in the set system
    #[arg(short, long)]
    number: Option<i32>,

    /// Name of file to save the result
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Args)]
struct PartitionArgs {
    /// Name of the algorithm used to create the partition
    #[arg(short, long)]
    algo: Algo,

    /// Number of parts in the partition
    #[arg(short, long)]
    tpart: i32,

    /// File containing the set system to partition
    #[arg(short, long)]
    setsystem: String,

    /// Number of rounds to simulate disctance function in the parallel algorithm
    #[arg(short, long)]
    warmup: Option<i32>,

    /// Name of file to save the result
    #[arg(short, long)]
    output: Option<String>,

    /// Write the result stats to a file
    #[arg(short, long)]
    results: Option<String>,
}

#[derive(Args)]
struct IntersectionsArgs {
    /// Set system file
    #[arg(short, long)]
    setsystem: String,

    /// Partition file
    #[arg(short, long)]
    partition: String,
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
    let cli = Cli::parse();
    match &cli.command {
        Commands::Generate(args) => {
            let d: usize = args.dimension;
            let n: i32;
            let ss: SetSystem;
            match args.sstype.as_str() {
                "grid" => {
                    n = args.number.expect("n is required for the grid set system");
                    ss = SetSystem::grid(n, d);
                }
                "rhs" => {
                    n = args.number.expect("n is required for the grid set system");
                    ss = SetSystem::rhs(n, ((n as f32) * (n as f32).ln()).floor() as i32, d);
                }
                _ => {
                    eprintln!("Invalid set system type: {}", args.sstype);
                    std::process::exit(1);
                }
            }
            match &args.output {
                None => (),
                Some(x) => ss.to_file(x.as_str()),
            }
        }
        Commands::Partition(args) => {
            let ss: SetSystem = SetSystem::from_file(&args.setsystem);
            let t = args.tpart;
            let n = ss.points.len();
            let res: SetSystem;
            let time: Duration;
            let mut warmup = (n as f32).sqrt() as i32;
            match args.algo {
                Algo::Min => {
                    (res, time) = algos::part_min(&ss, t);
                }
                Algo::AO => match args.warmup {
                    None => (res, time) = algos::part_at_once(&ss, t, t / 10),
                    Some(w) => {
                        (res, time) = algos::part_at_once(&ss, t, w);
                        warmup = w
                    }
                },
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
            match &args.results {
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
                        ss.points[0].coordinates.len(),
                        0,
                        intersections.iter().max().expect(""),
                        mean(&intersections),
                        intersections.iter().min().expect(""),
                        warmup,
                        time.as_secs_f64(),
                        0,
                        0
                    ) {
                        eprintln!("Couldn't write to file: {}", e);
                    }
                }
            }
            match &args.output {
                None => (),
                Some(x) => res.to_file(x.as_str()),
            }
        }
        Commands::Intersections(args) => {
            let ss: SetSystem = SetSystem::from_file(&args.setsystem);
            let part: SetSystem = SetSystem::from_file(&args.partition);
            let intersections = intersections(&part.sets, &ss.sets);
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
        }
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
