use crate::ss::{Point, Set, SetSystem};
use indicatif::ProgressBar;
use rand::Rng;
use rayon::prelude::*;
use std::time::Instant;

pub fn intersects<const D: usize>((i, j): (usize, usize), s: &Set<D>) -> bool {
    s.points[i] != s.points[j]
}

pub fn part_min<const D: usize>(ss: &SetSystem<D>, t: i32) -> SetSystem<D> {
    //SetSystem constants
    let n = ss.points.len();
    let m = ss.sets.len();
    let mut rng = rand::thread_rng();

    let now = Instant::now();
    let (_pin, _pout, sin, sout) = ss.build_adjacency();
    let elapsed = now.elapsed();
    println!("Building adjacency took: {:.3?}", elapsed);

    let now = Instant::now();
    //Build result points and sets vectors
    let mut res_sets: Vec<Set<D>> = Vec::new();
    let mut res_points: Vec<Point<D>> = Vec::new();
    for p in ss.points.iter() {
        res_points.push(p.clone());
    }

    //List all points not yet in  a part
    let mut available_pts: Vec<bool> = vec![true; n];
    let mut pt_weight: Vec<u128> = vec![0; n];
    let mut set_weight: Vec<u128> = vec![0; m];

    let bar = ProgressBar::new(t as u64);
    //Part building
    for i in 0..t - 1 {
        bar.inc(1);
        let mut part: Vec<bool> = vec![false; n];

        //sets_weight to normalize in the potential function
        //let mut sets_weight: u128 = 0;
        // for j in 1..m {
        //     sets_weight += 1 << set_weight[j];
        // }

        //let mut part_weight: u128 = 0;
        let mut intersect_part: Vec<bool> = vec![false; m];

        let mut temp: Vec<usize> = Vec::new();
        for i in available_pts.iter().enumerate() {
            if let (j, true) = i {
                temp.push(j);
                pt_weight[j] = 0;
            }
        }
        let start = temp[rng.gen_range(0..temp.len())];
        part[start] = true;
        available_pts[start] = false;
        for j in 0..m {
            if ss.sets[j].points[start] {
                for k in sout[j].iter() {
                    pt_weight[*k] += 1 << set_weight[j];
                }
            } else {
                for k in sin[j].iter() {
                    pt_weight[*k] += 1 << set_weight[j];
                }
            }
        }
        for _ in 1..(n as i32 / t) {
            let mut min = n + 1;
            for l in 0..n {
                if available_pts[l] {
                    if min == n + 1 || pt_weight[l] < pt_weight[min] {
                        min = l;
                    } else if min == n + 1
                        || pt_weight[l] == pt_weight[min] && rng.gen::<f32>() > 0.5
                    {
                        min = l;
                    }
                }
            }
            part[min] = true;
            available_pts[min] = false;

            for j in 0..m {
                if !intersect_part[j] && intersects((start, min), &ss.sets[j]) {
                    if ss.sets[j].points[start] {
                        for x in sout[j].iter() {
                            if available_pts[*x] && intersects((start, *x), &ss.sets[j]) {
                                pt_weight[*x] -= 1 << set_weight[j];
                            }
                        }
                    } else {
                        for x in sin[j].iter() {
                            if available_pts[*x] && intersects((start, *x), &ss.sets[j]) {
                                pt_weight[*x] -= 1 << set_weight[j];
                            }
                        }
                    }
                    intersect_part[j] = true;
                    set_weight[j] += 1;
                }
            }
        }
        res_sets.push(Set {
            points: part,
            index: (i + 1) as usize,
        });
    }
    bar.inc(1);
    let mut part: Vec<bool> = vec![false; n];
    for x in available_pts.iter().enumerate() {
        if let (p, true) = x {
            part[p] = true
        }
    }
    res_sets.push(Set {
        points: part,
        index: t as usize,
    });
    bar.finish();

    let elapsed = now.elapsed();
    println!("Elapsed: {:.3?}", elapsed);
    SetSystem {
        points: res_points,
        sets: res_sets,
    }
}

pub fn part_at_once<const D: usize>(ss: &SetSystem<D>, t: i32, k: i32) -> SetSystem<D> {
    //SetSystem constants
    let n = ss.points.len();
    let m = ss.sets.len();
    let mut rng = rand::thread_rng();

    let now = Instant::now();
    let (_pin, _pout, sin, sout) = ss.build_adjacency();
    let elapsed = now.elapsed();
    println!("Building adjacency took: {:.3?}", elapsed);

    let now = Instant::now();
    //Build result points and sets vectors
    let mut res_sets: Vec<Set<D>> = Vec::new();
    let mut res_points: Vec<Point<D>> = Vec::new();
    for p in ss.points.iter() {
        res_points.push(p.clone());
    }

    //List all points not yet in  a part
    let mut available_pts: Vec<bool> = vec![true; n];
    let mut set_weight: Vec<u128> = vec![0; m];

    let bar = ProgressBar::new(t as u64);
    //Part building
    for i in 0..t - 1 {
        bar.inc(1);
        let mut part: Vec<bool> = vec![false; n];

        let mut temp: Vec<usize> = Vec::new();
        for l in available_pts.iter().enumerate() {
            if let (j, true) = l {
                temp.push(j);
            }
        }
        let start = temp[rng.gen_range(0..temp.len())];
        part[start] = true;
        available_pts[start] = false;
        let distances = distance(&ss, &available_pts, start, k, &set_weight, &sin, &sout);
        let mut tosort: Vec<(usize, &u128)> = Vec::new();
        for x in distances.iter().enumerate() {
            if available_pts[x.0] {
                tosort.push(x);
            }
        }
        tosort.sort_by(|a, b| a.1.cmp(&b.1));
        for l in 0..(n as i32 / t - 1) as usize {
            part[tosort[l].0] = true;
            available_pts[tosort[l].0] = false;
        }
        set_weight = (0..m)
            .into_par_iter()
            .map(|j| {
                update_weight(
                    &ss.sets[j],
                    &sout[j],
                    &sin[j],
                    set_weight[j],
                    &tosort,
                    n,
                    t,
                    start,
                )
            })
            .collect();
        res_sets.push(Set {
            points: part,
            index: (i + 1) as usize,
        });
    }
    bar.inc(1);
    let mut part: Vec<bool> = vec![false; n];
    for x in available_pts.iter().enumerate() {
        if let (p, true) = x {
            part[p] = true
        }
    }
    res_sets.push(Set {
        points: part,
        index: t as usize,
    });
    bar.finish();

    let elapsed = now.elapsed();
    println!("Elapsed: {:.3?}", elapsed);
    SetSystem {
        points: res_points,
        sets: res_sets,
    }
}

fn update_weight<const D: usize>(
    s: &Set<D>,
    sout: &Vec<usize>,
    sin: &Vec<usize>,
    initial_weight: u128,
    tosort: &Vec<(usize, &u128)>,
    n: usize,
    t: i32,
    start: usize,
) -> u128 {
    let mut res = initial_weight;
    let mut test = false;
    if s.points[start] {
        for k in sout.iter() {
            if test {
                break;
            }
            for l in 0..(n as i32 / t - 1) as usize {
                if *k == tosort[l].0 {
                    res += 1;
                    test = true;
                    break;
                }
            }
        }
    } else {
        for k in sin.iter() {
            if test {
                break;
            }
            for l in 0..(n as i32 / t - 1) as usize {
                if *k == tosort[l].0 {
                    res += 1;
                    test = true;
                    break;
                }
            }
        }
    }
    res
}

fn distance<const D: usize>(
    ss: &SetSystem<D>,
    available: &Vec<bool>,
    start: usize,
    k: i32,
    sets_weight: &Vec<u128>,
    sin: &Vec<Vec<usize>>,
    sout: &Vec<Vec<usize>>,
) -> Vec<u128> {
    let n = ss.points.len();
    let mut res = vec![0; n];
    for i in 0..n {
        if available[i] {
            res[i] = 1;
        }
    }
    for _ in 0..k {
        let s = exponential_pick(sets_weight);
        if ss.sets[s].points[start] {
            for i in sout[s].iter() {
                if available[*i] {
                    res[*i] += 1 << sets_weight[s];
                }
            }
        } else {
            for i in sin[s].iter() {
                if available[*i] {
                    res[*i] += 1 << sets_weight[s];
                }
            }
        }
    }
    res
}

fn exponential_pick(w: &Vec<u128>) -> usize {
    let mut total: u128 = 0;
    let mut rng = rand::thread_rng();
    for i in 0..w.len() {
        total += 1 << w[i];
    }
    let stop_at = rng.gen_range(0..total);
    let mut partial_sum = 0;
    let mut i: usize = 0;
    while i < w.len() && (partial_sum + (1 << w[i])) < stop_at {
        partial_sum += 1 << w[i];
        i += 1;
    }
    i
}
