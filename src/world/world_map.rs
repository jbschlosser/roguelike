extern crate astar;
extern crate rand;

use random::RandomTable;
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

        let tiles: Vec<_> = ::std::iter::repeat(Terrain::Nothing)
            .take((width * height) as usize)
            .map(|terrain| Tile::new(terrain))
            .collect();

        let mut world = WorldMap { width: width, height: height, tiles: tiles };

        // Generate random features.
        let feature_shapes: Vec<(Box<Fn(&mut R) -> FeatureBuilder>, u32)> =
            vec![
                (Box::new(|rng: &mut R| {
                    let i = rng.gen_range::<i32>(3, 15);
                    let j = rng.gen_range::<i32>(3, 15);
                    FeatureBuilder::room(i, j)
                }), 1)
            ];
        let feature_table = RandomTable::new(feature_shapes);
        let mut features: Vec<Feature> = Vec::new();
        'outer: while features.len() < 12 {
            let feature_builder = feature_table.generate(rng);
            let feature_x = rng.gen_range::<i32>(0, width);
            let feature_y = rng.gen_range::<i32>(0, height);
            let feature = feature_builder
                .vert_align(VerticalAlignment::Top)
                .horiz_align(HorizontalAlignment::Left)
                .location(Location::new(feature_x, feature_y))
                .build();

            // Check if it fits in the world.
            for &(loc, _) in feature.iter() {
                if loc.x < 0 || loc.y < 0 || loc.x >= width || loc.y >= height {
                    println!("Collides with map edges!");
                    continue 'outer;
                }
            }

            // Check if it collides with anything in the world.
            for &(loc, _) in feature.iter() {
                if world.get_tile(loc).terrain != Terrain::Nothing {
                    println!("Collides with something in the world!");
                    continue 'outer;
                }
            }

            // Draw feature.
            for &(loc, terrain) in feature.iter() {
                world.get_tile_mut(loc).terrain = terrain;
            }

            // Draw path from this to another random feature.
            let mut should_add = true;
            if features.len() > 0 {
                println!("Features: {}", features.len());
                let this_wall = feature.walls().random(rng);
                let other_feature = features.iter().random(rng);
                let other_wall = other_feature.walls().random(rng);
                if world.get_tile(*this_wall).terrain != Terrain::Wall || world.get_tile(*other_wall).terrain != Terrain::Wall {
                    for &(loc, _) in feature.iter() {
                        world.get_tile_mut(loc).terrain = Terrain::Nothing;
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
                        for &(loc, _) in feature.iter() {
                            world.get_tile_mut(loc).terrain = Terrain::Nothing;
                        }
                    }
                }
            }
            if should_add { features.push(feature); }
        }

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
        let starting_loc = *features.iter().random(rng).floors().random(rng);
        let tiles2: Vec<_> = ::std::iter::repeat(Terrain::Nothing)
            .take((width * height) as usize)
            .map(|terrain| Tile::new(terrain))
            .collect();
        let mut world2 = WorldMap {width: width, height: height, tiles: tiles2};
        for i in width-10..width-5 {
            for j in 0..height {
                world2.get_tile_mut(Location::new(i, j)).terrain = Terrain::Debug;
            }
        }

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
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Location {
    pub x: i32,
    pub y: i32
}

impl Location {
    pub fn new(x: i32, y: i32) -> Self {
        Location {x: x, y: y}
    }
    pub fn manhattan(&self, other: &Location) -> i32 {
        let total_x = if self.x > other.x { self.x - other.x } else { other.x - self.x };
        let total_y = if self.y > other.y { self.y - other.y } else { other.y - self.y };

        total_x + total_y
    }
}

impl ::std::fmt::Debug for Location {
    fn fmt(&self, f: &mut ::std::fmt::Formatter) ->
        Result<(), ::std::fmt::Error> {
        f.write_fmt(format_args!("({}, {})", self.x, self.y))
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Terrain {
    Debug,
    Nothing,
    Floor,
    Wall
}

#[derive(Copy, Clone, Debug)]
pub struct Entity {
    id: u64
}

pub struct Tile {
    pub terrain: Terrain,
    pub entities: Vec<Entity>
}

impl Tile {
    pub fn new(terrain: Terrain) -> Self {
        Tile {terrain: terrain, entities: Vec::new()}
    }
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

// GENERATION STUFF.
// A feature in the world, consisting of some arrangement of terrain.
// Features consist of relative coordinates; they can be placed at any
// arbitrary location.
#[derive(Clone, Debug)]
struct Feature {
    components: Vec<(Location, Terrain)>
}

impl Feature {
    pub fn new(components: Vec<(Location, Terrain)>) -> Self {
        Feature { components: components }
    }
    pub fn overlaps(&self, other: &Feature) -> bool {
        for a in self.components.iter() {
            for b in other.components.iter() {
                if a == b { return true; }
            }
        }

        return false;
    }
    /*pub fn width(&self) -> i32 {
        if self.components.len() == 0 {
            return 0;
        }

        self.components.iter().map(|c| c.0.x).max().unwrap() -
            self.components.iter().map(|c| c.0.x).min().unwrap() + 1
    }
    pub fn height(&self) -> i32 {
        if self.components.len() == 0 {
            return 0;
        }

        self.components.iter().map(|c| c.0.y).max().unwrap() -
            self.components.iter().map(|c| c.0.y).min().unwrap() + 1
    }*/
    pub fn iter(&self) -> ::std::slice::Iter<(Location, Terrain)> {
        self.components.iter()
    }
    pub fn walls<'a>(&'a self) -> Box<Iterator<Item=&'a Location> + 'a> {
        Box::new(self.components.iter()
            .filter(|c| c.1 == Terrain::Wall)
            .map(|c| &c.0))
    }
    pub fn floors<'a>(&'a self) -> Box<Iterator<Item=&'a Location> + 'a> {
        Box::new(self.components.iter()
            .filter(|c| c.1 == Terrain::Floor)
            .map(|c| &c.0))
    }
}

#[derive(Clone, Copy, Debug)]
enum HorizontalAlignment {
    Left,
    Center,
    Right
}

#[derive(Clone, Copy, Debug)]
enum VerticalAlignment {
    Top,
    Center,
    Bottom
}

// Build features! Take the raw feature shape and translate it
// according to the given alignment and absolute location.
#[derive(Clone, Debug)]
struct FeatureBuilder {
    components: Vec<(Location, Terrain)>,
    location: Location,
    horiz_align: HorizontalAlignment,
    vert_align: VerticalAlignment
}

impl FeatureBuilder {
    pub fn new(components: Vec<(Location, Terrain)>) -> Self {
        assert!(components.len() > 0);
        FeatureBuilder {
            components: components,
            location: Location::new(0, 0),
            horiz_align: HorizontalAlignment::Left,
            vert_align: VerticalAlignment::Top
        }
    }
    pub fn room(width: i32, height: i32) -> Self {
        let mut components = Vec::new();
        for x in 0..width {
            for y in 0..height {
                let terrain =
                    if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                        Terrain::Wall
                    } else {
                        Terrain::Floor
                    };
                components.push((Location::new(x, y), terrain));
            }
        }

        FeatureBuilder::new(components)
    }
    pub fn location(mut self, loc: Location) -> Self {
        self.location = loc;
        self
    }
    pub fn horiz_align(mut self, align: HorizontalAlignment) -> Self {
        self.horiz_align = align;
        self
    }
    pub fn vert_align(mut self, align: VerticalAlignment) -> Self {
        self.vert_align = align;
        self
    }
    pub fn build(&self) -> Feature {
        let horiz = match self.horiz_align {
            HorizontalAlignment::Left => {
                self.location.x - Self::calc_min_x(&self.components)
            },
            HorizontalAlignment::Center => {
                self.location.x - (Self::calc_min_x(&self.components) +
                    (Self::calc_max_x(&self.components) -
                    Self::calc_min_x(&self.components) + 1) / 2)
            },
            HorizontalAlignment::Right => {
                self.location.x - Self::calc_max_x(&self.components)
            }
        };
        let vert = match self.vert_align {
            VerticalAlignment::Top => {
                self.location.y - Self::calc_min_y(&self.components)
            },
            VerticalAlignment::Center => {
                self.location.y - (Self::calc_min_y(&self.components) +
                    (Self::calc_max_y(&self.components) -
                    Self::calc_min_y(&self.components) + 1) / 2)
            },
            VerticalAlignment::Bottom => {
                self.location.y - Self::calc_max_y(&self.components)
            }
        };

        let comps = self.components.iter()
            .map(|c| {
                (Location::new(c.0.x + horiz, c.0.y + vert), c.1)
            })
            .collect();

        Feature::new(comps)
    }

    fn calc_min_x(components: &[(Location, Terrain)]) -> i32 {
        components.iter().map(|c| c.0.x).min().unwrap()
    }
    fn calc_max_x(components: &[(Location, Terrain)]) -> i32 {
        components.iter().map(|c| c.0.x).max().unwrap()
    }
    fn calc_min_y(components: &[(Location, Terrain)]) -> i32 {
        components.iter().map(|c| c.0.y).min().unwrap()
    }
    fn calc_max_y(components: &[(Location, Terrain)]) -> i32 {
        components.iter().map(|c| c.0.y).max().unwrap()
    }
}

// Trait to extend iterators to provide a random function.
trait IterRandomExt<T> {
    fn random<R: Rng>(&mut self, rng: &mut R) -> T;
}

impl<I> IterRandomExt<I::Item> for I where I: Iterator, I::Item: Clone {
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

#[test]
fn test_feature_size() {
    let feature = Feature::new(vec![
        (Location::new(0, 0), Terrain::Wall),
        (Location::new(1, 0), Terrain::Wall),
        (Location::new(1, 1), Terrain::Wall)]);
    assert_eq!(feature.width(), 2);
    assert_eq!(feature.height(), 2);
}

#[test]
fn test_build_feature() {
    // Set up some feature shape:
    //
    // ....
    // .##.
    // .##.
    // .#..
    // ....
    let comps = vec![
        (Location::new(1, 1), Terrain::Wall),
        (Location::new(2, 1), Terrain::Wall),
        (Location::new(1, 2), Terrain::Wall),
        (Location::new(2, 2), Terrain::Wall),
        (Location::new(1, 3), Terrain::Wall)
    ];

    // Place top left at (2,3).
    //
    // .....
    // .....
    // .....
    // ..##.
    // ..##.
    // ..#..
    // .....
    assert_eq!(FeatureBuilder::new(comps.clone())
        .vert_align(VerticalAlignment::Top)
        .horiz_align(HorizontalAlignment::Left)
        .location(Location::new(2, 3))
        .build().components,
        vec![
            (Location::new(2, 3), Terrain::Wall),
            (Location::new(3, 3), Terrain::Wall),
            (Location::new(2, 4), Terrain::Wall),
            (Location::new(3, 4), Terrain::Wall),
            (Location::new(2, 5), Terrain::Wall)
        ]);

    // Place bottom right at (5,2).
    //
    // ....##.
    // ....##.
    // ....#..
    // .......
    assert_eq!(FeatureBuilder::new(comps)
        .vert_align(VerticalAlignment::Bottom)
        .horiz_align(HorizontalAlignment::Right)
        .location(Location::new(5, 2))
        .build().components,
        vec![
            (Location::new(4, 0), Terrain::Wall),
            (Location::new(5, 0), Terrain::Wall),
            (Location::new(4, 1), Terrain::Wall),
            (Location::new(5, 1), Terrain::Wall),
            (Location::new(4, 2), Terrain::Wall)
        ]);

    // Set up a square feature shape.
    //
    // ###
    // ###
    // ###
    let square = vec![
        (Location::new(0, 0), Terrain::Wall),
        (Location::new(1, 0), Terrain::Wall),
        (Location::new(2, 0), Terrain::Wall),
        (Location::new(0, 1), Terrain::Wall),
        (Location::new(1, 1), Terrain::Wall),
        (Location::new(2, 1), Terrain::Wall),
        (Location::new(0, 2), Terrain::Wall),
        (Location::new(1, 2), Terrain::Wall),
        (Location::new(2, 2), Terrain::Wall),
    ];

    // Place center at (4, 1).
    //
    // ...###.
    // ...###.
    // ...###.
    // .......
    assert_eq!(FeatureBuilder::new(square)
        .vert_align(VerticalAlignment::Center)
        .horiz_align(HorizontalAlignment::Center)
        .location(Location::new(4, 1))
        .build().components,
        vec![
            (Location::new(3, 0), Terrain::Wall),
            (Location::new(4, 0), Terrain::Wall),
            (Location::new(5, 0), Terrain::Wall),
            (Location::new(3, 1), Terrain::Wall),
            (Location::new(4, 1), Terrain::Wall),
            (Location::new(5, 1), Terrain::Wall),
            (Location::new(3, 2), Terrain::Wall),
            (Location::new(4, 2), Terrain::Wall),
            (Location::new(5, 2), Terrain::Wall)
        ]);
}
