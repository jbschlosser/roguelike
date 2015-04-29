#![feature(box_syntax)]

extern crate astar;
extern crate rand;
extern crate tcod;

use rand::{Rng, StdRng};
use tcod::{Console, BackgroundFlag};
use tcod::RootInitializer;
use tcod::input::Key::Special;
use tcod::input::KeyCode::{Up, Down, Left, Right, Escape};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Location {
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Terrain {
    Debug,
    Nothing,
    Floor,
    Wall
}

#[derive(Copy, Clone, Debug)]
struct Entity {
    id: u64
}

struct Tile {
    pub terrain: Terrain,
    pub entities: Vec<Entity>
}

impl Tile {
    pub fn new(terrain: Terrain) -> Self {
        Tile {terrain: terrain, entities: Vec::new()}
    }
}

struct WorldMap {
    width: i32,
    height: i32,
    tiles: Vec<Tile>
}

struct Room {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    locs: Vec<Location>
}

impl Room {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        assert!(x >= 0 && y >= 0 && width > 0 && height > 0);
        let mut locs: Vec<Location> = Vec::with_capacity((width * height) as usize);
        for i in x..x+width {
            for j in y..y+height {
                locs.push(Location::new(i, j));
            }
        }

        Room {x: x, y: y, width: width, height: height, locs: locs}
    }
    pub fn overlaps(&self, other: &Room) -> bool {
        let (xmin1, xmax1, xmin2, xmax2) = (self.x, self.x + self.width,
            other.x, other.x + other.width);
        let (ymin1, ymax1, ymin2, ymax2) = (self.y, self.y + self.height,
            other.y, other.y + other.height);
        (xmax1 >= xmin2) && (xmax2 >= xmin1) && (ymax1 >= ymin2) && (ymax2 >= ymin1)
    }
    pub fn get_walls<'a>(&'a self) -> Box<Iterator<Item=&'a Location> + 'a> {
        Box::new(self.locs.iter().filter(move |loc| loc.x == self.x || loc.y == self.y ||
            loc.x == self.x + self.width - 1 || loc.y == self.y + self.height - 1))
    }
    pub fn get_floors<'a>(&'a self) -> Box<Iterator<Item=&'a Location> + 'a> {
        Box::new(self.locs.iter().filter(move |loc| loc.x > self.x && loc.y > self.y &&
            loc.x < self.x + self.width - 1 && loc.y < self.y + self.height - 1))
    }
}

impl WorldMap {
    pub fn generate<R: Rng>(rng: &mut R, width: i32, height: i32) -> (Self, Location) {
        assert!(width > 0);
        assert!(height > 0);

        let tiles: Vec<_> = std::iter::repeat(Terrain::Nothing)
            .take((width * height) as usize)
            .map(|terrain| Tile::new(terrain))
            .collect();

        let mut world = WorldMap { width: width, height: height, tiles: tiles };

        // Generate rooms.
        let mut rooms: Vec<Room> = Vec::new();
        for _ in 0..60 {
            let room_width = rng.gen_range::<i32>(3, 15);
            let room_height = rng.gen_range::<i32>(3, 15);
            let room_x = rng.gen_range::<i32>(0, width - room_width);
            let room_y = rng.gen_range::<i32>(0, height - room_height);
            let room = Room::new(room_x, room_y, room_width, room_height);
            let mut available = true;
            for chosen in rooms.iter() {
                if chosen.overlaps(&room) {
                    available = false;
                    break;
                }
            }
            if available {
                //println!("{}x{} @ {}x{}", room_width, room_height, room_x, room_y);
                rooms.push(room);
            } else {
                //println!("Couldn't fit it");
            }
        }

        // Draw rooms.
        let mut floor_locs = Vec::new();
        let mut wall_locs = Vec::new();
        for room in rooms.iter() {
            for wall in room.get_walls() {
                println!("wall: {:?}", wall);
                world.get_tile_mut(*wall).terrain = Terrain::Wall;
                wall_locs.push(wall);
            }

            for floor in room.get_floors() {
                world.get_tile_mut(*floor).terrain = Terrain::Floor;
                floor_locs.push(*floor);
            }
        }

        // Draw paths between rooms.
        for _ in 0..1 {
            // Dig out walls and find path.
            let wall1 = wall_locs[rng.gen_range::<usize>(0, wall_locs.len() - 1)];
            let wall2 = wall_locs[rng.gen_range::<usize>(0, wall_locs.len() - 1)];
            world.get_tile_mut(*wall1).terrain = Terrain::Nothing;
            world.get_tile_mut(*wall2).terrain = Terrain::Nothing;
            println!("Searching for path from {:?} to {:?}...", wall1, wall2);
            match astar::astar(ConnectRooms::new(&world, *wall1, *wall2)) {
                Some(path) => {
                    for loc in path.iter() {
                        world.get_tile_mut(*loc).terrain = Terrain::Debug;
                    }
                },
                None => { println!("Failed to find path"); }
            }
        }

        // Pick a random floor tile to start on.
        let starting_loc = floor_locs[rng.gen_range::<usize>(0, floor_locs.len() - 1)];

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

struct TileIterator<'a> {
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

fn main() {
    let width = 80;
    let height = 50;
    let mut console = RootInitializer::new()
        .size(width, height)
        .title("Roguelike")
        .init();

    let mut rng = StdRng::new().unwrap();
    let (world, starting_loc) = WorldMap::generate(&mut rng, width, height);

    let mut location = starting_loc;
    while !console.window_closed() {
        // Draw world.
        console.clear();
        for (tile, location) in world.tiles() {
            match tile.terrain {
                Terrain::Floor => {
                    console.put_char(location.x, location.y, '.', BackgroundFlag::Set);
                },
                Terrain::Wall => {
                    console.put_char(location.x, location.y, '#', BackgroundFlag::Set);
                },
                Terrain::Nothing => {
                    console.put_char(location.x, location.y, ' ', BackgroundFlag::Set);
                },
                Terrain::Debug => {
                    console.put_char(location.x, location.y, '^', BackgroundFlag::Set);
                }
            }
        }

        // Draw character.
        console.put_char(location.x, location.y, '@', BackgroundFlag::Set);
        console.flush();

        // Check for keypress.
        let keypress = console.wait_for_keypress(true);
        if keypress.pressed {
            let new_loc = match keypress.key {
                Special(Escape) => break,
                Special(Up) => {
                    if location.y > 0 {
                        Location::new(location.x, location.y - 1)
                    } else { location }
                },
                Special(Down) => Location::new(location.x, location.y + 1),
                Special(Left) => {
                    if location.x > 0 {
                        Location::new(location.x - 1, location.y)
                    } else { location }
                },
                Special(Right) => Location::new(location.x + 1, location.y),
                _ => location
            };
            match world.get_tile(new_loc).terrain {
                Terrain::Floor => location = new_loc,
                Terrain::Debug => location = new_loc,
                Terrain::Wall => {},
                Terrain::Nothing => {}
            }
        }
    }
}
