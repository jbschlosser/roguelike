extern crate rand;
extern crate tcod;

use rand::{Rng, StdRng};
use tcod::{Console, BackgroundFlag};
use tcod::RootInitializer;
use tcod::input::Key::Special;
use tcod::input::KeyCode::{Up, Down, Left, Right, Escape};

#[derive(Copy, Clone, Debug)]
struct Location {
    pub x: i32,
    pub y: i32
}

impl Location {
    pub fn new(x: i32, y: i32) -> Self {
        Location {x: x, y: y}
    }
}

#[derive(Copy, Clone, Debug)]
enum Terrain {
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
    height: i32
}

impl Room {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Room {x: x, y: y, width: width, height: height}
    }
    pub fn overlaps(&self, other: &Room) -> bool {
        let (xmin1, xmax1, xmin2, xmax2) = (self.x, self.x + self.width,
            other.x, other.x + other.width);
        let (ymin1, ymax1, ymin2, ymax2) = (self.y, self.y + self.height,
            other.y, other.y + other.height);
        (xmax1 >= xmin2) && (xmax2 >= xmin1) && (ymax1 >= ymin2) && (ymax2 >= ymin1)
    }
}

impl WorldMap {
    pub fn generate<R: Rng>(rng: &mut R, width: i32, height: i32) -> (Self, Location) {
        assert!(width > 0);
        assert!(height > 0);

        let mut tiles = Vec::with_capacity((width * height) as usize);
        for _ in 0..width*height {
            tiles.push(Tile::new(Terrain::Nothing));
        }

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
            // Horizontal walls.
            for x in room.x+1..room.x+room.width-1 {
                let wall1 = Location::new(x, room.y);
                let wall2 = Location::new(x, room.y+room.height-1);
                world.get_tile_mut(wall1).terrain = Terrain::Wall;
                world.get_tile_mut(wall2).terrain = Terrain::Wall;
                wall_locs.push(wall1);
                wall_locs.push(wall2);
            }
            // Vertical walls.
            for y in room.y+1..room.y+room.height-1 {
                let wall1 = Location::new(room.x, y);
                let wall2 = Location::new(room.x+room.width-1, y);
                world.get_tile_mut(wall1).terrain = Terrain::Wall;
                world.get_tile_mut(wall2).terrain = Terrain::Wall;
                wall_locs.push(wall1);
                wall_locs.push(wall2);
            }
            // Corners.
            world.get_tile_mut(Location::new(room.x, room.y)).terrain = Terrain::Wall;
            world.get_tile_mut(Location::new(room.x, room.y+room.height-1)).terrain =
                Terrain::Wall;
            world.get_tile_mut(Location::new(room.x+room.width-1, room.y)).terrain =
                Terrain::Wall;
            world.get_tile_mut(Location::new(room.x+room.width-1,
                room.y+room.height-1)).terrain = Terrain::Wall;
            // Floors.
            for x in room.x+1..room.x+room.width-1 {
                for y in room.y+1..room.y+room.height-1 {
                    let loc = Location::new(x, y);
                    world.get_tile_mut(loc).terrain = Terrain::Floor;
                    floor_locs.push(loc);
                }
            }
        }

        // Draw paths between rooms.
        for _ in 0..1 {
            let wall1 = wall_locs[rng.gen_range::<usize>(0, wall_locs.len() - 1)];
            let wall2 = wall_locs[rng.gen_range::<usize>(0, wall_locs.len() - 1)];
            let path = world.find_path(wall1, wall2);
            println!("Drawing path from {:?} to {:?}: {:?}", wall1, wall2, path);
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
    fn find_path(&self, start: Location, end: Location) -> Option<Vec<Location>> {
        // TODO: Implement priority-queue based A* algorithm.
        /*let mut potential = Vec::new();
        potential.push(vec![start]);
        while potential.len() > 0 && start != end {
            let next_try = potential.pop();
            let adjacent = self.get_adjacent(start);
            for loc in adjacent {
                match self.get_tile(loc).terrain {
                    Terrain::Floor => {
                        println!("Adjacent: {:?}", loc);
                    },
                    _ => {}
                }
            }
        }

        return potential.pop();*/

        return None;
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
        console.clear();

        // Draw world.
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
                }
            }
        }

        // Draw character.
        console.put_char(location.x, location.y, '@', BackgroundFlag::Set);

        console.flush();
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
                Terrain::Wall => {},
                Terrain::Nothing => {}
            }
        }
    }
}
