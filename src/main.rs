use ss::SetSystem;

mod ss;

fn main() {
    const D: usize = 2;
    const N: i32 = 10;
    let ss: SetSystem<D> = SetSystem::grid(N);
    let (pin, pout, sin, sout) = ss.clone().build_adjacency();
    for p in ss.points {
        println!("{:?}", p.coordinates);
    }
    println!("");
    for s in ss.sets {
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

fn greedy<const D: usize>(ss: SetSystem<D>) -> SetSystem<D> {
    ss
}
