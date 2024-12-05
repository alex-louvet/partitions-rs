use ss::Point;
use ss::Set;
use ss::SetSystem;

mod ss;

fn main() {
    const D: usize = 2;
    const N: i32 = 10;
    let ss: SetSystem<D> = SetSystem::grid(N);
    let (pin, pout, sin, sout) = ss.build_adjacency();
    let ss2 = greedy(&ss, 10);
    for p in ss2.points {
        println!("{:?}", p.coordinates);
    }
    println!("");
    for s in ss2.sets {
        println!("{:?}", s.points);
    }
    println!("");
    for l in sin {
        println!("{:?}", l);
    }
    println!("");
    for l in sout {
        println!("{:?}", l);
    }
}

fn greedy<const D: usize>(ss: &SetSystem<D>, t: i32) -> SetSystem<D> {
    //SetSystem constants
    let n = ss.points.len();
    let m = ss.sets.len();

    //Build result points and sets vectors
    let mut res_sets: Vec<Set<D>> = Vec::new();
    let mut res_points: Vec<Point<D>> = Vec::new();
    for p in ss.points.iter() {
        res_points.push(p.clone());
    }

    //List all points not yet in  a part
    let mut available_pts: Vec<bool> = vec![true; n];
    let mut pt_weight: Vec<u128> = vec![0; n];

    //Part building
    for i in 0..t {
        let mut part: Vec<bool> = vec![true; n];

        //sets_weight to normalize in the potential function
        let mut sets_weight: u128 = 0;
        for s in ss.sets.iter() {
            sets_weight += 1 << s.weight;
        }

        let mut part_weight: u128 = 0;
        let mut intersect_partition: Vec<bool> = vec![false; n];

        let mut temp: Vec<usize> = Vec::new();
        for i in available_pts.iter().enumerate() {
            match i {
                (j, true) => temp.push(j),
                (_, false) => (),
            }
        }
    }

    return SetSystem {
        points: res_points,
        sets: res_sets,
    };
}
