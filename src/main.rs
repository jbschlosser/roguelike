extern crate rand;
extern crate tcod;
extern crate world;

use rand::StdRng;
use tcod::{Console, BackgroundFlag, RootInitializer};
use tcod::input::Key::{Special, Printable};
use tcod::input::KeyCode::{Up, Down, Left, Right, Escape};
use world::{WorldMap, Terrain, Location};

fn main() {
    let width = 150; //80;
    let height = 100; //50;
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
        for tile in world.tiles() {
            match tile.terrain {
                Terrain::Floor => {
                    console.put_char(tile.loc.x, tile.loc.y, '.', BackgroundFlag::Set);
                },
                Terrain::Wall => {
                    console.put_char(tile.loc.x, tile.loc.y, '#', BackgroundFlag::Set);
                },
                Terrain::Nothing => {
                    console.put_char(tile.loc.x, tile.loc.y, ' ', BackgroundFlag::Set);
                },
                Terrain::Debug => {
                    console.put_char(tile.loc.x, tile.loc.y, '^', BackgroundFlag::Set);
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
                Printable('y') => {
                    if location.x > 0 && location.y > 0 {
                        Location::new(location.x - 1, location.y - 1)
                    } else { location }
                },
                Printable('u') => {
                    if location.y > 0 {
                        Location::new(location.x + 1, location.y - 1)
                    } else { location }
                },
                Printable('b') => {
                    if location.x > 0 {
                        Location::new(location.x - 1, location.y + 1)
                    } else { location }
                },
                Printable('n') => {
                    Location::new(location.x + 1, location.y + 1)
                },
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
