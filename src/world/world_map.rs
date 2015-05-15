extern crate rand;

use random::RandomTable;
use tile::{Tile, Terrain, Location};
use feature::{Feature, VerticalAlignment, HorizontalAlignment};
use self::rand::{Rng};

pub struct WorldMap {
    width: i32,
    height: i32,
    tiles: Vec<Tile>
}

impl WorldMap {
    pub fn generate<R: Rng>(rng: &mut R, width: i32, height: i32) ->
        (Self, Location)
    {
        assert!(width > 0);
        assert!(height > 0);

        let mut world = {
            let mut tiles = Vec::new();
            for j in 0..height {
                for i in 0..width {
                    tiles.push(Tile::new(Location::new(i, j),
                        Terrain::Nothing));
                }
            }

            WorldMap { width: width, height: height, tiles: tiles }
        };

        // Populate the random feature generator table.
        let feature_table = RandomTable::new(
            vec![
                (Box::new(|rng: &mut R| {
                    let i = rng.gen_range::<i32>(2, 12);
                    let j = rng.gen_range::<i32>(2, 12);
                    Feature::room(i, j)
                }), 1),
                (Box::new(|rng: &mut R| {
                    let r = rng.gen_range::<i32>(2, 12);
                    Feature::diamond_room(r)
                }), 1),
                (Box::new(|rng: &mut R| {
                    let r = rng.gen_range::<i32>(2, 12);
                    Feature::circle_room(r)
                }), 1)
            ]);

        // Place first feature somewhere in the middle.
        // NOTE: Magic do/while syntax.
        let mut first_feature = feature_table.generate(rng);
        while {
            let feature_x = rng.gen_range::<i32>(
                width / 2 - 7, width / 2 + 7);
            let feature_y = rng.gen_range::<i32>(
                height / 2 - 7, height / 2 + 7);
            let feat_loc = Location::new(feature_x, feature_y);
            first_feature = first_feature.place(VerticalAlignment::Center,
                HorizontalAlignment::Center, feat_loc);

            !world.can_fit(&first_feature)
        } {}

        // Draw first feature.
        world.draw_feature(&first_feature);
        world.surround_floors_with_walls();

        // Try drawing more features each connected by hallways.
        for _ in 0..300 {
            // Pick a random wall in the world.
            let rand_wall = world.tiles()
                .filter(|t| t.terrain == Terrain::Wall)
                .random(rng)
                .clone();

            // Draw a hallway attached to it.
            world.get_tile_mut(rand_wall.loc).terrain = Terrain::Nothing;
            let hallway_len = rng.gen_range::<i32>(5, 15);
            let mut orientations = vec![
                (VerticalAlignment::Center, HorizontalAlignment::Left),
                (VerticalAlignment::Center, HorizontalAlignment::Right),
                (VerticalAlignment::Top, HorizontalAlignment::Center),
                (VerticalAlignment::Bottom, HorizontalAlignment::Center)
            ];
            rng.shuffle(&mut orientations);
            let mut orientation = None;
            for &(vert, horiz) in orientations.iter() {
                let is_horiz = vert == VerticalAlignment::Center;
                let hallway = Feature::hallway(hallway_len, is_horiz)
                    .place(vert, horiz, rand_wall.loc);
                if world.can_fit(&hallway) {
                    orientation = Some((vert, horiz));
                    break;
                }
            }
            match orientation {
                Some((vert, horiz)) => {
                    let is_horiz = vert == VerticalAlignment::Center;
                    let hallway = Feature::hallway(hallway_len, is_horiz)
                        .place(vert, horiz, rand_wall.loc);
                    world.draw_feature(&hallway);

                    // Determine how the feature should be placed to connect
                    // with the hallway.
                    let feat_orientation =
                        if vert == VerticalAlignment::Center {
                            (vert, horiz, Location::new(
                                if horiz == HorizontalAlignment::Left {
                                    rand_wall.loc.x + hallway_len - 1
                                } else {
                                    rand_wall.loc.x - hallway_len + 1
                                }, rand_wall.loc.y
                            ))
                        } else {
                            (vert, horiz, Location::new(
                                rand_wall.loc.x,
                                if vert == VerticalAlignment::Top {
                                    rand_wall.loc.y + hallway_len - 1
                                } else {
                                    rand_wall.loc.y - hallway_len + 1
                                }
                            ))
                        };

                    // Generate a random feature attached to the hallway.
                    world.get_tile_mut(feat_orientation.2).terrain =
                        Terrain::Nothing;
                    let feature = feature_table.generate(rng)
                        .place(feat_orientation.0, feat_orientation.1,
                            feat_orientation.2);
                    if world.can_fit(&feature) {
                        // Draw the feature.
                        world.draw_feature(&feature);
                        world.get_tile_mut(feat_orientation.2).terrain =
                            Terrain::Floor;
                        world.surround_floors_with_walls();
                    } else {
                        world.undraw_feature(&hallway);
                        world.get_tile_mut(rand_wall.loc).terrain =
                            Terrain::Wall;
                    }
                },
                None => {
                    world.get_tile_mut(rand_wall.loc).terrain = Terrain::Wall;
                    continue;
                }
            }
        }

        // Pick a random floor in the first room to start on.
        let starting_loc = *first_feature.floors().random(rng);

        (world, starting_loc)
    }
    pub fn tiles(&self) -> ::std::slice::Iter<Tile> {
        self.tiles.iter()
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
        for i in -1..2 {
            for j in -1..2 {
                if !(i == 0 && j == 0) {
                    let new_x = loc.x + i;
                    let new_y = loc.y + j;
                    if new_x >= 0 && new_x < self.width &&
                        new_y >= 0 && new_y < self.height
                    {
                        adjacent.push(Location::new(new_x, new_y));
                    }
                }
            }
        }

        return adjacent;
    }
    fn can_fit(&self, feature: &Feature) -> bool {
        // Check if it fits in the world.
        // NOTE: Keep 1 space away from the edges as a hack to
        // make sure all floors are surrounded by walls.
        for tile in feature.iter() {
            if tile.loc.x <= 0 || tile.loc.y <= 0 ||
                tile.loc.x >= self.width - 1 || tile.loc.y >= self.height - 1 {
                return false;
            }

            if self.get_tile(tile.loc).terrain != Terrain::Nothing {
                return false;
            }
        }

        true
    }
    fn draw_feature(&mut self, feature: &Feature) {
        for tile in feature.iter() {
            self.get_tile_mut(tile.loc).terrain = tile.terrain;
        }
    }
    fn undraw_feature(&mut self, feature: &Feature) {
        for tile in feature.iter() {
            self.get_tile_mut(tile.loc).terrain = Terrain::Nothing;
        }
    }
    fn surround_floors_with_walls(&mut self) {
        let mut make_wall_locs =
            Vec::with_capacity((self.width * self.height) as usize);
        for tile in self.tiles.iter().filter(|t| t.terrain == Terrain::Floor) {
            for adj in self.get_adjacent(tile.loc).iter()
                .filter(|l| self.get_tile(**l).terrain == Terrain::Nothing)
            {
                make_wall_locs.push(*adj);
            }
        }
        for loc in make_wall_locs.iter() {
            self.get_tile_mut(*loc).terrain = Terrain::Wall;
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub struct Entity {
    id: u64
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
/*struct NeighborIterator {
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

impl<'a> astar::SearchProblem<Location, i32, NeighborIterator>
    for ConnectRooms<'a>
{
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
}*/
