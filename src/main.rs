extern crate rand;
extern crate tcod;

use rand::{Rand, Rng, StdRng};
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
    pub fn generate<R: Rng>(rng: &mut R, width: i32, height: i32) -> Self {
        assert!(width > 0);
        assert!(height > 0);

        let mut tiles = Vec::with_capacity((width * height) as usize);
        for i in 0..width*height {
            tiles.push(Tile::new(Terrain::Floor));
        }

        let mut world = WorldMap {width: width, height: height, tiles: tiles};

        // Generate rooms.
        let mut rooms: Vec<Room> = Vec::new();
        for i in 0..60 {
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
                println!("{}x{} @ {}x{}", room_width, room_height, room_x, room_y);
                rooms.push(room);
            } else {
                println!("Couldn't fit it");
            }
        }

        for room in rooms.iter() {
            for x in room.x..room.x+room.width {
                world.get_tile_mut(Location::new(x, room.y)).terrain = Terrain::Wall;
                world.get_tile_mut(Location::new(x, room.y+room.height-1)).terrain =
                    Terrain::Wall;
            }
            for y in room.y..room.y+room.height {
                world.get_tile_mut(Location::new(room.x, y)).terrain = Terrain::Wall;
                world.get_tile_mut(Location::new(room.x+room.width-1, y)).terrain =
                    Terrain::Wall;
            }
        }

        world
    }
    pub fn tiles(&self) -> TileIterator {
        TileIterator::new(&self.tiles, self.width)
    }
    pub fn get_tile(&self, loc: Location) -> &Tile {
        let index = (loc.y * self.width + loc.x) as usize;
        assert!(index >= 0 && index < self.tiles.len());
        &self.tiles[index]
    }
    pub fn get_tile_mut(&mut self, loc: Location) -> &mut Tile {
        let index = (loc.y * self.width + loc.x) as usize;
        assert!(index >= 0 && index < self.tiles.len());
        &mut self.tiles[index]
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
    let WIDTH = 80;
    let HEIGHT = 50;
    let mut console = RootInitializer::new()
        .size(WIDTH, HEIGHT)
        .title("Roguelike")
        .init();

    let mut rng = StdRng::new().unwrap();
    let mut world = WorldMap::generate(&mut rng, WIDTH, HEIGHT);

    let mut location = Location {x: 0, y: 0};
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
                Special(Up) => Location::new(location.x, location.y - 1),
                Special(Down) => Location::new(location.x, location.y + 1),
                Special(Left) => Location::new(location.x - 1, location.y),
                Special(Right) => Location::new(location.x + 1, location.y),
                _ => location
            };
            match world.get_tile(new_loc).terrain {
                Terrain::Floor => location = new_loc,
                Terrain::Wall => {},
            }
        }
    }
}
