extern crate astar;
extern crate rand;

use random::RandomTable;
use tile::{Tile, Terrain, Location};
use feature::{Feature, FeatureBuilder, VerticalAlignment, HorizontalAlignment};
use self::rand::{Rng};

pub struct WorldMap {
    width: i32,
    height: i32,
    tiles: Vec<Tile>
}

impl WorldMap {
    pub fn generate<R: Rng>(rng: &mut R, width: i32, height: i32) -> (Self, Location) {
        assert!(width > 0);
        assert!(height > 0);

        let tiles = {
            let mut tiles_temp = Vec::new();
            for i in 0..width {
                for j in 0..height {
                    tiles_temp.push(Tile::new(Location::new(i, j), Terrain::Nothing));
                }
            }
            tiles_temp
        };

        let mut world = WorldMap { width: width, height: height, tiles: tiles };

        // Generate random features.
        let feature_shapes: Vec<(Box<Fn(&mut R) -> FeatureBuilder>, u32)> =
            vec![
                (Box::new(|rng: &mut R| {
                    let i = rng.gen_range::<i32>(3, 15);
                    let j = rng.gen_range::<i32>(3, 15);
                    FeatureBuilder::room(i, j)
                }), 1),
                (Box::new(|rng: &mut R| {
                    let r = rng.gen_range::<i32>(3, 15);
                    FeatureBuilder::diamond_room(r)
                }), 1),
                (Box::new(|rng: &mut R| {
                    let r = rng.gen_range::<i32>(3, 15);
                    FeatureBuilder::circle_room(r)
                }), 1),
                /*(Box::new(|rng: &mut R| {
                    let l = rng.gen_range::<i32>(3, 15);
                    let is_horiz = rng.gen::<bool>();
                    FeatureBuilder::hallway(l, is_horiz)
                }), 5)*/
            ];
        let feature_table = RandomTable::new(feature_shapes);
        let mut features: Vec<Feature> = Vec::new();

        // Place first feature somewhere in the middle.
        // TODO: Check that the first feature fits!
        let feature_x = rng.gen_range::<i32>(width / 2 - 3, width / 2 + 3);
        let feature_y = rng.gen_range::<i32>(height / 2 - 3, height / 2 + 3);
        let first_feature = feature_table.generate(rng)
            .vert_align(VerticalAlignment::Center)
            .horiz_align(HorizontalAlignment::Center)
            .location(Location::new(feature_x, feature_y))
            .build();

        // Draw first feature.
        for tile in first_feature.iter() {
            world.get_tile_mut(tile.loc).terrain = tile.terrain;
        }

        /*'outer: while features.len() < 12 {
            let feature_builder = feature_table.generate(rng);
            let feature_x = rng.gen_range::<i32>(0, width);
            let feature_y = rng.gen_range::<i32>(0, height);
            let feature = feature_builder
                .vert_align(VerticalAlignment::Top)
                .horiz_align(HorizontalAlignment::Left)
                .location(Location::new(feature_x, feature_y))
                .build();

            // Check if it fits in the world.
            for tile in feature.iter() {
                if tile.loc.x < 0 || tile.loc.y < 0 || tile.loc.x >= width || tile.loc.y >= height {
                    println!("Collides with map edges!");
                    continue 'outer;
                }
            }

            // Check if it collides with anything in the world.
            for tile in feature.iter() {
                if world.get_tile(tile.loc).terrain != Terrain::Nothing {
                    println!("Collides with something in the world!");
                    continue 'outer;
                }
            }

            // Draw feature.
            for tile in feature.iter() {
                world.get_tile_mut(tile.loc).terrain = tile.terrain;
            }

            // Draw path from this to another random feature.
            let mut should_add = true;
            if features.len() > 0 {
                println!("Features: {}", features.len());
                let this_wall = feature.walls().random(rng);
                let other_feature = features.iter().random(rng);
                let other_wall = other_feature.walls().random(rng);
                if world.get_tile(*this_wall).terrain != Terrain::Wall || world.get_tile(*other_wall).terrain != Terrain::Wall {
                    for tile in feature.iter() {
                        world.get_tile_mut(tile.loc).terrain = Terrain::Nothing;
                    }
                    continue 'outer;
                }

                // Dig out walls and find path.
                world.get_tile_mut(*this_wall).terrain = Terrain::Nothing;
                world.get_tile_mut(*other_wall).terrain = Terrain::Nothing;
                println!("Searching for path from {:?} to {:?}...", this_wall, other_wall);
                match astar::astar(ConnectRooms::new(&world, *this_wall, *other_wall)) {
                    Some(path) => {
                        println!("Path found!");
                        for loc in path.iter() {
                            world.get_tile_mut(*loc).terrain = Terrain::Floor;
                        }
                        world.get_tile_mut(*this_wall).terrain = Terrain::Debug;
                        world.get_tile_mut(*other_wall).terrain = Terrain::Debug;
                    },
                    None => {
                        println!("Failed to find path");

                        // Put the other wall back.
                        world.get_tile_mut(*other_wall).terrain = Terrain::Wall;

                        // Undraw feature.
                        should_add = false;
                        for tile in feature.iter() {
                            world.get_tile_mut(tile.loc).terrain = Terrain::Nothing;
                        }
                    }
                }
            }
            if should_add { features.push(feature); }
        }*/

        // Draw features.
        /*for feature in features.iter() {
            for wall in feature.walls() {
                world.get_tile_mut(*wall).terrain = Terrain::Wall;
            }

            for floor in feature.floors() {
                world.get_tile_mut(*floor).terrain = Terrain::Floor;
            }
        }*/

        // Draw paths between rooms.
        /*for _ in 0..20 {
            // Pick two random walls from two random rooms.
            let wall1 = features.iter().random(rng).walls().random(rng);
            let wall2 = features.iter().random(rng).walls().random(rng);

            // Dig out walls and find path.
            world.get_tile_mut(*wall1).terrain = Terrain::Nothing;
            world.get_tile_mut(*wall2).terrain = Terrain::Nothing;
            println!("Searching for path from {:?} to {:?}...", wall1, wall2);
            match astar::astar(ConnectRooms::new(&world, *wall1, *wall2)) {
                Some(path) => {
                    for loc in path.iter() {
                        world.get_tile_mut(*loc).terrain = Terrain::Debug;
                    }
                },
                None => {
                    println!("Failed to find path");
                    world.get_tile_mut(*wall1).terrain = Terrain::Wall;
                    world.get_tile_mut(*wall2).terrain = Terrain::Wall;
                }
            }
        }*/

        // Pick a random floor in a random room to start on.
        //let starting_loc = *features.iter().random(rng).floors().random(rng);
        let starting_loc = Location::new(0, 0);
        /*let tiles2: Vec<_> = ::std::iter::repeat(Terrain::Nothing)
            .take((width * height) as usize)
            .map(|terrain| Tile::new(terrain))
            .collect();
        let mut world2 = WorldMap {width: width, height: height, tiles: tiles2};
        for i in width-10..width-5 {
            for j in 0..height {
                world2.get_tile_mut(Location::new(i, j)).terrain = Terrain::Debug;
            }
        }*/

        (world, starting_loc)
    }
    pub fn tiles(&self) -> TileIterator {
        TileIterator::new(&self.tiles, self.width)
    }
    pub fn get_tile(&self, loc: Location) -> &Tile {
        let index = (loc.y * self.width + loc.x) as usize;
        assert!(index < self.tiles.len());
        &self.tiles[index]
    }
    pub fn get_tile_mut(&mut self, loc: Location) -> &mut Tile {
        let index = (loc.y * self.width + loc.x) as usize;
        assert!(index < self.tiles.len());
        &mut self.tiles[index]
    }
    fn get_adjacent(&self, loc: Location) -> Vec<Location> {
        let mut adjacent = Vec::new();
        if loc.x > 0 { adjacent.push(Location::new(loc.x - 1, loc.y)); }
        if loc.y > 0 { adjacent.push(Location::new(loc.x, loc.y - 1)); }
        if loc.x < self.width - 1 { adjacent.push(Location::new(loc.x + 1, loc.y)); }
        if loc.y < self.height - 1 { adjacent.push(Location::new(loc.x, loc.y + 1)); }

        return adjacent;
    }
    fn can_fit(&self, feature: &Feature) -> bool {
        // Check if it fits in the world.
        for tile in feature.iter() {
            if tile.loc.x < 0 || tile.loc.y < 0 ||
                tile.loc.x >= self.width || tile.loc.y >= self.height {
                return false;
            }

            if self.get_tile(tile.loc).terrain != Terrain::Nothing {
                return false;
            }
        }

        true
    }
}


#[derive(Copy, Clone, Debug)]
pub struct Entity {
    id: u64
}

pub struct TileIterator<'a> {
    tiles: &'a [Tile],
    width: i32,
    curr: usize
}

impl<'a> TileIterator<'a> {
    pub fn new(tiles: &'a [Tile], width: i32) -> Self {
        TileIterator {tiles: tiles, width: width, curr: 0}
    }
}

impl<'a> Iterator for TileIterator<'a> {
    type Item = (&'a Tile, Location);

    fn next(&mut self) -> Option<(&'a Tile, Location)> {
        if self.curr < self.tiles.len() {
            let this = self.curr as i32;
            self.curr += 1;
            Some((
                &self.tiles[this as usize],
                Location {
                    x: this % self.width,
                    y: this / self.width
                }))
        } else { None }
    }
}

// Trait to extend iterators to provide a random function.
trait IterRandomExt<T> {
    fn random<R: Rng>(&mut self, rng: &mut R) -> T;
}

impl<I: Iterator> IterRandomExt<I::Item> for I where I::Item: Clone {
    fn random<R: Rng>(&mut self, rng: &mut R) -> I::Item {
        let elements: Vec<_> = self.collect();
        assert!(elements.len() > 0);
        let random = rng.gen_range::<usize>(0, elements.len());
        elements[random].clone()
    }
}

// Iterates through neighbors; used for A* algorithm.
struct NeighborIterator {
    adjacent: Vec<Location>,
    current: usize
}

impl NeighborIterator {
    pub fn new(world: &WorldMap, loc: Location) -> Self {
        let adjacent = world.get_adjacent(loc).iter()
            .map(|x| *x)
            .filter(|loc| world.get_tile(*loc).terrain == Terrain::Nothing)
            .collect();

        NeighborIterator { adjacent: adjacent, current: 0 }
    }
}

impl Iterator for NeighborIterator {
    type Item = (Location, i32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.adjacent.len() {
            self.current += 1;
            Some((self.adjacent[self.current - 1], 1))
        } else {
            None
        }
    }
}

// Search problem for connecting rooms with A* algorithm.
struct ConnectRooms<'a> {
    world: &'a WorldMap,
    start: Location,
    end: Location
}

impl<'a> ConnectRooms<'a> {
    pub fn new(world: &'a WorldMap, start: Location, end: Location) -> Self {
        ConnectRooms { world: world, start: start, end: end }
    }
}

impl<'a> astar::SearchProblem<Location, i32, NeighborIterator> for ConnectRooms<'a> {
    fn start(&self) -> Location {
        self.start
    }
    fn is_end(&self, loc: &Location) -> bool {
        *loc == self.end
    }
    fn heuristic(&self, loc: &Location) -> i32 {
        loc.manhattan(&self.end)
    }
    fn neighbors(&self, at: &Location) -> NeighborIterator {
        NeighborIterator::new(&self.world, *at)
    }
}

