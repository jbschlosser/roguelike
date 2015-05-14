extern crate rand;

use self::rand::Rng;

// A random table that can be used to generate items in a weighted way.
pub struct RandomTable<T, R: Rng> {
    // Contains generation functions + min, max random values
    // that correspond to picking each function.
    table: Vec<(Box<Fn(&mut R) -> T>, (u32, u32))>,
    max: u32
}

impl<T, R: Rng> RandomTable<T, R> {
    pub fn new(items_with_weights: Vec<(Box<Fn(&mut R) -> T>, u32)>) -> Self {
        let mut sum = 0;
        let mut table = Vec::with_capacity(items_with_weights.len());
        for (item, weight) in items_with_weights {
            table.push((item, (sum, sum + weight - 1)));
            sum += weight;
        }

        RandomTable {table: table, max: sum}
    }
    pub fn generate(&self, rng: &mut R) -> T {
        let rand_num = rng.gen_range::<u32>(0, self.max);
        for entry in self.table.iter() {
            if rand_num >= (entry.1).0 && rand_num <= (entry.1).1 {
                return entry.0(rng)
            }
        }

        panic!("BUG: Random table was built incorrectly.");
    }
}
