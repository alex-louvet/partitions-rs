use num_integer::Roots;
use rand::seq::SliceRandom;
use rulinalg::matrix::decomposition::PartialPivLu;
use rulinalg::matrix::Matrix;
use std::{fs, io::BufWriter, io::Write};

#[derive(Debug, Clone)]
pub struct Point<const D: usize> {
    pub coordinates: [f32; D],
    pub index: usize,
}

// impl<'a, const D: usize> Point<'a, D> {
//     pub fn build_adjacency(&mut self, sets: Vec<&'a Set<D>>) {
//         for s in sets.iter() {
//             if s.points[self.index] {
//                 self.sets_in.push(s);
//             } else {
//                 self.not_in.push(s);
//             }
//         }
//     }
// }

#[derive(Debug)]
pub struct Set<const D: usize> {
    pub points: Vec<bool>,
    // pub weight: i32,
    pub index: usize,
}

// impl<const D: usize> Set<D> {
//     pub fn increase(&mut self) {
//         self.weight += 1;
//     }

//     pub fn decrease(&mut self) {
//         self.weight -= 1;
//     }

//     pub fn reset(&mut self) {
//         self.weight = 0;
//     }
// }

// impl<'a, const D: usize> Set<'a, D> {
//     pub fn build_adjacency(&mut self, pts: Vec<&'a Point<D>>) {
//         for i in 0..pts.len() {
//             if self.points[i] {
//                 self.points_ref.push(pts[i]);
//             } else {
//                 self.points_ref_complement.push(pts[i]);
//             }
//         }
//     }
// }

pub struct SetSystem<const D: usize> {
    pub points: Vec<Point<D>>,
    pub sets: Vec<Set<D>>,
}

impl<const D: usize> SetSystem<D> {
    pub fn grid(n: i32) -> SetSystem<D> {
        let mut points = Vec::new();
        for i in 0..n {
            let mut temp = [0.; D];
            for j in 0..D {
                temp[j] = rand::random::<f32>();
            }
            points.push(Point {
                coordinates: temp,
                index: i as usize,
            });
        }
        let mut sets = Vec::new();
        let mut index: usize = 0;
        for d in 0..D {
            for i in 0..n.nth_root(D as u32) {
                let mut temp: Vec<bool> = vec![false; n as usize];
                let mut temp2: Vec<bool> = vec![false; n as usize];
                for p in points.iter() {
                    temp[p.index] =
                        p.coordinates[d] * f32::powf(n as f32, 1.0 / (D as f32)) > i as f32;
                    temp2[p.index] =
                        p.coordinates[d] * f32::powf(n as f32, 1.0 / (D as f32)) > i as f32;
                }
                sets.push(Set {
                    points: temp,
                    index,
                });
                index += 1;
                sets.push(Set {
                    points: temp2,
                    index,
                });
                index += 1;
            }
        }
        SetSystem { points, sets }
    }

    pub fn rhs(n: i32, m: i32) -> SetSystem<D> {
        let mut points = Vec::new();
        for i in 0..n {
            let mut temp = [0.; D];
            for j in 0..D {
                temp[j] = rand::random::<f32>();
            }
            points.push(Point {
                coordinates: temp,
                index: i as usize,
            });
        }
        let mut sets = Vec::new();
        for j in 0..(m / 2) as usize {
            let sample: Vec<_> = points.choose_multiple(&mut rand::thread_rng(), D).collect();
            let mut v = Vec::new();
            for s in sample.iter() {
                v.extend(s.coordinates);
            }
            let mat = Matrix::new(D, D, v);
            let b = vector![1.0;D];
            let lu = PartialPivLu::decompose(mat).expect("Matrix is invertible");
            let y = lu.solve(b).expect("Matrix is invertible.");
            let mut set = vec![false; points.len()];
            let mut set_c = vec![false; points.len()];
            for (i, p) in points.iter().enumerate() {
                let mut temp = 0.0;
                for k in 0..D {
                    temp += p.coordinates[k] as f32 * y[k]
                }
                if temp > 1.0 {
                    set[i] = true;
                } else {
                    set_c[i] = true;
                }
            }
            sets.push(Set {
                points: set,
                index: 2 * j,
            });
            sets.push(Set {
                points: set_c,
                index: 2 * j + 1,
            });
        }
        SetSystem { points, sets }
    }

    pub fn build_adjacency(
        &self,
    ) -> (
        Vec<Vec<usize>>,
        Vec<Vec<usize>>,
        Vec<Vec<usize>>,
        Vec<Vec<usize>>,
    ) {
        let mut sets_adj = vec![Vec::<usize>::new(); self.sets.len()];
        let mut points_adj = vec![Vec::<usize>::new(); self.points.len()];
        let mut sets_adj_complement = vec![Vec::<usize>::new(); self.sets.len()];
        let mut points_adj_complement = vec![Vec::<usize>::new(); self.points.len()];
        for p in self.points.iter() {
            for s in self.sets.iter() {
                if s.points[p.index] {
                    sets_adj[s.index].push(p.index);
                    points_adj[p.index].push(s.index);
                } else {
                    sets_adj_complement[s.index].push(p.index);
                    points_adj_complement[p.index].push(s.index);
                }
            }
        }

        (
            points_adj,
            points_adj_complement,
            sets_adj,
            sets_adj_complement,
        )
    }

    pub fn to_file(&self, filename: &str) -> () {
        let mut file = BufWriter::new(fs::File::create(filename).expect("Fail to create file"));
        for x in self.points.iter() {
            for c in x.coordinates.iter() {
                file.write(c.to_string().as_bytes()).expect("Fail to write");
                file.write(b",").expect("Fail to write");
            }
            file.write(b"\n").expect("Fail to write");
        }
        file.write(b"sets\n").expect("Fail to write");
        for x in self.sets.iter() {
            for c in x.points.iter() {
                file.write((*c as i8).to_string().as_bytes())
                    .expect("Fail to write");
                file.write(b",").expect("Fail to write");
            }
            file.write(b"\n").expect("Fail to write");
        }
    }
}
