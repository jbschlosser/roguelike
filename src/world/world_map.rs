extern crate itertools;
extern crate nalgebra;
extern crate rand;

use random::{RandomTable, IterRandomExt};
use tile::{Tile, Terrain, Location};
use feature::{Feature, VerticalAlignment, HorizontalAlignment};
use std::hash::Hash;
use std::collections::HashMap;
use std::collections::HashSet;
use self::itertools::Itertools;
use self::nalgebra::DMat;
use self::rand::{Rng};

/// A pathfinding map structure. A Dijkstra map lets you run pathfinding from
/// any graph node it covers towards or away from the target nodes of the map.
/// Currently the structure only supports underlying graphs with a fixed grid graph
/// where the neighbors of each node must be the adjacent grid cells of that
/// node.
pub struct Dijkstra<'a> {
    weights: HashMap<Location, u32>,
    world: &'a WorldMap
}

impl<'a> Dijkstra<'a> {
    /// Create a new Dijkstra map up to limit distance from goals.
    pub fn new(world: &'a WorldMap, goals: Vec<Location>, limit: u32) -> Self {
        assert!(goals.len() > 0);

        let mut weights = HashMap::new();
        let mut edge = HashSet::new();

        for n in goals.into_iter() {
            edge.insert(n);
        }

        for dist in 0..(limit) {
            for n in edge.iter() {
                weights.insert(n.clone(), dist);
            }

            let mut new_edge = HashSet::new();
            for n in edge.iter() {
                for m in world.get_adjacent(*n, true).into_iter()
                    .filter(|l| world.get_tile(*l).terrain == Terrain::Floor) {
                    if !weights.contains_key(&m) {
                        new_edge.insert(m);
                    }
                }
            }

            edge = new_edge;
        }

        Dijkstra {
            weights: weights,
            world: world
        }
    }

    /// Return the neighbors of a cell (if any), sorted from downhill to
    /// uphill.
    pub fn sorted_neighbors(& self, node: &Location) -> Vec<Location> {
        let mut ret = Vec::new();
        for n in self.world.get_adjacent(*node, true).iter() {
            if let Some(w) = self.weights.get(n) {
                ret.push((w, n.clone()));
            }
        }
        ret.sort_by(|&(w1, _), &(w2, _)| w1.cmp(w2));
        ret.into_iter().map(|(_, n)| n).collect()
    }
}

#[derive(Eq, PartialEq, Clone, Hash)]
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

        // Clear out walls with three or more floors adjacent
        // to them in cardinal directions.
        let make_floor_locs: Vec<Location> = world.tiles()
            .filter(|t| t.terrain == Terrain::Wall &&
                world.get_adjacent(t.loc, false).iter()
                    .filter(|a| world.get_tile(**a).terrain == Terrain::Floor)
                    .count() >= 3)
            .map(|t| t.loc.clone())
            .collect();

        for loc in make_floor_locs.iter() {
            world.get_tile_mut(*loc).terrain = Terrain::Floor;
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
    pub fn create_dijkstra_map(&self, goals: &Vec<Location>) -> DMat<i32> {
        // Create a matrix for each location in the map, with each
        // score set very high.
        let mut dmap = DMat::from_elem(self.height as usize,
            self.width as usize, ::std::i32::MAX - 2);

        // Set score of goals to 0.
        goals.iter().foreach(|g| dmap[g.to_matrix_index()] = 0);

        // Update matrix until no changes have been made.
        let mut changed = true;
        while changed {
            changed = false;
            self.tiles()
                .filter(|t| t.terrain == Terrain::Floor)
                .foreach(|t| {
                    let smallest = self.get_adjacent(t.loc, true).iter()
                        .map(|l| dmap[l.to_matrix_index()])
                        .min()
                        .unwrap();

                    if dmap[t.loc.to_matrix_index()] >= smallest + 2 {
                        dmap[t.loc.to_matrix_index()] = smallest + 1;
                        changed = true;
                    }
                });
        }

        dmap
    }
    fn get_adjacent(&self, loc: Location, with_diag: bool) -> Vec<Location> {
        let mut adjacent = Vec::new();
        for i in -1..2 {
            for j in -1..2 {
                if i != 0 || j != 0 {
                    if !with_diag && (i != 0 && j != 0) {
                        continue;
                    }
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
            for adj in self.get_adjacent(tile.loc, true).iter()
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
