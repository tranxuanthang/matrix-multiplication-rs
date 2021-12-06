use std::thread;
use std::fmt;
use rand::Rng;
use std::time::Instant;
use std::sync::mpsc;
use std::sync::Arc;

#[derive(Clone)]
struct Matrix {
    width: usize,
    height: usize,
    cells: Vec<i32>
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, cell) in self.cells.iter().enumerate() {
            if index % self.width == 0 {
                write!(f, "> ")?;
            }

            write!(f, "{:>6}", cell)?;

            if (index + 1) % self.width == 0 {
                write!(f, "\n")?;
            } else {
                write!(f, " ")?;
            }
        }

        Ok(())
    }
}

impl Matrix {
    fn new(width: usize, height: usize, cells: Vec<i32>) -> Option<Matrix> {
        let size: usize = width * height;
        let cells = cells;

        if cells.len() != size {
            return None
        }

        Some(Matrix {
            width,
            height,
            cells
        })
    }

    fn get(&self, x: usize, y: usize) -> Option<i32> {
        let size: usize = self.width * self.height;
        if x * y > size {
            return None
        }

        Some(self.cells[y * self.width + x])
    }

    fn set(&mut self, x: usize, y: usize, value: i32) -> Option<i32> {
        let size: usize = self.width * self.height;
        if x * y > size {
            return None
        }

        let cells = &mut self.cells;

        let index = y * self.width + x;

        let value = std::mem::replace(&mut cells[index], value);

        Some(value)
    }

    fn mul(self, m: Matrix) -> Option<Matrix> {
        let m1 = self;
        let m2 = m;

        if m1.width != m2.height {
            return None
        }

        let mut m = Matrix::new(m1.height, m2.width, vec![0; m1.height * m2.width])?;
        for i in 0..m.width {
            for j in 0..m.height {
                let mut cell = 0;
                for k in 0..m1.width {
                    cell += m1.get(k, i).unwrap() * m2.get(j, k).unwrap();
                }
                m.set(i, j, cell);
            }
        }

        Some(m)
    }

    fn mul_mt(self, m: Matrix) -> Option<Matrix> {
        let m1 = self;
        let m2 = m;

        if m1.width != m2.height {
            return None
        }

        let mut m = Matrix::new(m1.height, m2.width, vec![0; m1.height * m2.width])?;

        let mut thread_count = m.width;
        if thread_count > 12 {
            thread_count = 12;
        }

        let th_cols = m.width / thread_count;
        let th_cols_left = m.width % thread_count;

        let (tx, rx) = mpsc::channel();

        let m1_arc = Arc::new(m1);
        let m2_arc = Arc::new(m2);

        let m_height = m.height;

        for th_index in 0..thread_count {
            let tx_clone = tx.clone();
            let m1 = Arc::clone(&m1_arc);
            let m2 = Arc::clone(&m2_arc);

            thread::spawn(move || {
                let i_start = th_index * th_cols;
                let mut i_end = th_index * th_cols + th_cols;
                if th_index == thread_count - 1 {
                    // last thread
                    i_end = th_index * th_cols + th_cols + th_cols_left;
                }

                // println!("thread {} spawned. handle {} to {}", th_index, i_start, i_end);

                for i in i_start..i_end {
                    for j in 0..m_height {
                        let mut cell = 0;
                        for k in 0..m1.width {
                            cell += m1.get(k, i).unwrap() * m2.get(j, k).unwrap();
                        }
                        tx_clone.send((i, j, cell)).unwrap();
                    }
                }

                // println!("thread {} done", th_index);
            });
        }

        drop(tx);

        for received in rx {
            let (i, j, cell) = received;
            m.set(i, j, cell);
        }

        Some(m)
    }
}

fn generate_matrix(width: usize, height: usize) -> Matrix {

    let mut rng = rand::thread_rng();
    let vec: Vec<i32> = vec![0; width * height];
    let mut m = Matrix::new(width, height, vec).unwrap();

    for i in 0..width {
        for j in 0..height {
            let rand_value = rng.gen_range(-99..99);
            m.set(i, j, rand_value);
        }
    }

    m
}

fn main() {
    // let m1 = Matrix::new(3, 2, vec![1, 2, 3, 4, 5, 6]).unwrap();
    // let m2 = Matrix::new(2, 3, vec![7, 8, 9, 10, 11, 12]).unwrap();

    let height_m1 = 2000;
    let width_m1_height_m2 = 1000;
    let width_m2 = 4000;

    let m1 = generate_matrix(width_m1_height_m2, height_m1);
    let m2 = generate_matrix(width_m2, width_m1_height_m2);

    let m1_2 = m1.clone();
    let m2_2 = m2.clone();

    // println!("{}", &m1);
    // println!("{}", &m2);

    let now = Instant::now();
    let result_1 = m1.mul(m2).unwrap();
    let elapsed = now.elapsed();
    println!("Matrix multiplication single threaded took {:.2?}", elapsed);
    // println!("{}", &m);

    let now = Instant::now();
    let result_2 = m1_2.mul_mt(m2_2).unwrap();
    let elapsed = now.elapsed();
    println!("Matrix multiplication multi threaded took {:.2?}", elapsed);

    for i in 0..height_m1 {
        for j in 0..width_m2 {
            if result_1.get(i, j) != result_2.get(i, j) {
                panic!("Wrong ({} but {} at {},{})", result_1.get(i, j).unwrap(), result_1.get(i, j).unwrap(), i, j);
            }
        }
    }
}
