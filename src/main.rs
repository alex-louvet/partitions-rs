use ss::Set;
use ss::SetSystem;
use std::env;

mod algos;
mod ss;

fn main() {
    let args: Vec<String> = env::args().collect();
    const D: usize = 2;
    let n: i32 = args[1]
        .parse::<i32>()
        .expect("n can not be interpreted as i32");
    let t: i32 = args[2]
        .parse::<i32>()
        .expect("t can not be interpreted as i32");
    let ss: SetSystem<D> = SetSystem::grid(n);
    ss.to_file("ss.txt");
    let ss2 = algos::part_min(&ss, t);
    let intersections = intersections(&ss2.sets, &ss.sets);
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
    ss2.to_file("res.txt");
}

fn mean(v: &Vec<i32>) -> f32 {
    let mut sum = 0;
    for x in v.iter() {
        sum += x;
    }
    sum as f32 / v.len() as f32
}

fn intersections<const D: usize>(parts: &Vec<Set<D>>, ss: &Vec<Set<D>>) -> Vec<i32> {
    let n = parts[0].points.len();
    let mut inter = vec![0; ss.len()];
    for j in 0..ss.len() {
        for p in parts.iter() {
            let mut start: usize = n + 1;
            for i in 0..p.points.len() {
                if p.points[i] {
                    if start == n + 1 {
                        start = i;
                    } else if algos::intersects((start, i), &ss[j]) {
                        inter[j] += 1;
                        break;
                    }
                }
            }
        }
    }
    inter
}
