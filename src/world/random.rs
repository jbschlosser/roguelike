extern crate rand;

use self::rand::Rng;

// A random table that can be used to generate items in a weighted way.
pub struct RandomTable<T> where T: Clone {
    table: Vec<(T, (u32, u32))>, // min, max corresponding to this item
    max: u32
}

impl<T> RandomTable<T> where T: Clone {
    pub fn new(items_with_weights: Vec<(T, u32)>) -> Self {
        let mut sum = 0;
        let mut table = Vec::with_capacity(items_with_weights.len());
        for (item, weight) in items_with_weights {
            table.push((item, (sum, sum + weight - 1)));
            sum += weight;
        }

        RandomTable {table: table, max: sum}
    }
    pub fn generate<R: Rng>(&self, rng: &mut R) -> T {
        let rand_num = rng.gen_range::<u32>(0, self.max);
        println!("Generating from 0 to {}: {}", self.max, rand_num);
        for entry in self.table.iter() {
            if rand_num >= (entry.1).0 && rand_num <= (entry.1).1 {
                return entry.0.clone()
            }
        }

        panic!("BUG: Random table was built incorrectly.");
    }
}
