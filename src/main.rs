extern crate rand;
extern crate tcod;
extern crate world;

use rand::StdRng;
use std::collections::HashSet;
use tcod::input::Key::{Special, Printable};
use tcod::input::KeyCode::{Up, Down, Left, Right, Escape};
use tcod::{Console, BackgroundFlag, RootInitializer};
use world::{WorldMap, Terrain, Location};

fn explore(loc: Location, radius: i32, explored: &mut HashSet<Location>) {
    let center = Location::new(0, 0);
    for i in -radius..radius+1 {
        for j in -radius..radius+1 {
            let circle_loc = Location::new(i, j);
            let dist = circle_loc.euclidean(&center);
            if dist <= radius {
                explored.insert(Location::new(loc.x + circle_loc.x,
                    loc.y + circle_loc.y));
            }
        }
    }
}

fn main() {
    let width = 150; //80;
    let height = 100; //50;
    let player_radius = 5;
    let mut console = RootInitializer::new()
        .size(width, height)
        .title("Roguelike")
        .init();
    let mut rng = StdRng::new().unwrap();
    let (world, starting_loc) = WorldMap::generate(&mut rng, width, height);
    let mut explored = HashSet::new();
    let mut player_loc = starting_loc;
    explore(player_loc, player_radius, &mut explored);

    while !console.window_closed() {
        // Draw world.
        //console.clear();
        for tile in world.tiles() {
            match (tile.terrain, explored.contains(&tile.loc)) {
                (Terrain::Floor, true) => {
                    console.put_char(tile.loc.x, tile.loc.y, '.',
                        BackgroundFlag::Set);
                },
                (Terrain::Wall, true) => {
                    console.put_char(tile.loc.x, tile.loc.y, '#',
                        BackgroundFlag::Set);
                },
                (Terrain::Debug, true) => {
                    console.put_char(tile.loc.x, tile.loc.y, 'X',
                        BackgroundFlag::Set);
                },
                _ => {
                    console.put_char(tile.loc.x, tile.loc.y, ' ',
                        BackgroundFlag::Set);
                }
            }
        }

        // Draw character.
        console.put_char(player_loc.x, player_loc.y, '@', BackgroundFlag::Set);
        console.flush();

        // Check for keypress.
        let keypress = console.wait_for_keypress(true);
        if keypress.pressed {
            let new_loc = match keypress.key {
                Special(Escape) => break,
                Special(Up) => {
                    if player_loc.y > 0 {
                        Location::new(player_loc.x, player_loc.y - 1)
                    } else { player_loc }
                },
                Special(Down) => Location::new(player_loc.x, player_loc.y + 1),
                Special(Left) => {
                    if player_loc.x > 0 {
                        Location::new(player_loc.x - 1, player_loc.y)
                    } else { player_loc }
                },
                Special(Right) => Location::new(player_loc.x + 1, player_loc.y),
                Printable('y') => {
                    if player_loc.x > 0 && player_loc.y > 0 {
                        Location::new(player_loc.x - 1, player_loc.y - 1)
                    } else { player_loc }
                },
                Printable('u') => {
                    if player_loc.y > 0 {
                        Location::new(player_loc.x + 1, player_loc.y - 1)
                    } else { player_loc }
                },
                Printable('b') => {
                    if player_loc.x > 0 {
                        Location::new(player_loc.x - 1, player_loc.y + 1)
                    } else { player_loc }
                },
                Printable('n') => {
                    Location::new(player_loc.x + 1, player_loc.y + 1)
                },
                _ => player_loc
            };
            match world.get_tile(new_loc).terrain {
                Terrain::Floor => {
                    player_loc = new_loc;
                    explore(player_loc, player_radius, &mut explored);
                },
                Terrain::Debug => {
                    player_loc = new_loc;
                    explore(player_loc, player_radius, &mut explored);
                },
                Terrain::Wall => {},
                Terrain::Nothing => {}
            }
        }
    }
}
