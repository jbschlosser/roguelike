use tile::{Tile, Terrain, Location};

// GENERATION STUFF.
// A feature in the world, consisting of some arrangement of terrain.
// Features consist of relative coordinates; they can be placed at any
// arbitrary location.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Feature {
    tiles: Vec<Tile>
}

impl Feature {
    pub fn new(tiles: Vec<Tile>) -> Self {
        Feature { tiles: tiles }
    }
    pub fn room(width: i32, height: i32) -> Self {
        let mut tiles = Vec::new();
        for x in 0..width {
            for y in 0..height {
                tiles.push(Tile::new(Location::new(x, y), Terrain::Floor));
            }
        }

        Feature {tiles: tiles}
    }
    // TODO: Factor this out somehow; only the distance function
    // changes between circles and diamonds.
    pub fn diamond_room(radius: i32) -> Self {
        let center = Location::new(0, 0);
        let mut tiles = Vec::new();
        for i in -radius..radius+1 {
            for j in -radius..radius+1 {
                let loc = Location::new(i, j);
                let dist = loc.manhattan(&center);
                if dist <= radius {
                    tiles.push(Tile::new(loc, Terrain::Floor));
                }
            }
        }

        Feature {tiles: tiles}
    }
    pub fn circle_room(radius: i32) -> Self {
        let center = Location::new(0, 0);
        let mut tiles = Vec::new();
        for i in -radius..radius+1 {
            for j in -radius..radius+1 {
                let loc = Location::new(i, j);
                let dist = loc.euclidean(&center);
                if dist <= radius {
                    tiles.push(Tile::new(loc, Terrain::Floor));
                }
            }
        }

        Feature {tiles: tiles}
    }
    pub fn hallway(length: i32, is_horiz: bool) -> Self {
        assert!(length > 0);
        let mut tiles = Vec::new();
        if is_horiz {
            for i in 0..length {
                tiles.push(Tile::new(Location::new(i, 0), Terrain::Floor));
            }
        } else {
            for i in 0..length {
                tiles.push(Tile::new(Location::new(0, i), Terrain::Floor));
            }
        }

        Feature {tiles: tiles}
    }
    pub fn translate(&self, x: i32, y: i32) -> Self {
        Feature {
            tiles: self.tiles.iter()
                .map(|t| Tile::new(
                    Location::new(t.loc.x + x, t.loc.y + y), t.terrain))
                .collect()
        }
    }
    pub fn place(&self, vert_align: VerticalAlignment,
        horiz_align: HorizontalAlignment, loc: Location) -> Self
    {
        let horiz = match horiz_align {
            HorizontalAlignment::Left => {
                loc.x - self.min_x()
            },
            HorizontalAlignment::Center => {
                loc.x - (self.min_x() +
                    (self.max_x() -
                    self.min_x() + 1) / 2)
            },
            HorizontalAlignment::Right => {
                loc.x - self.max_x()
            }
        };
        let vert = match vert_align {
            VerticalAlignment::Top => {
                loc.y - self.min_y()
            },
            VerticalAlignment::Center => {
                loc.y - (self.min_y() +
                    (self.max_y() -
                    self.min_y() + 1) / 2)
            },
            VerticalAlignment::Bottom => {
                loc.y - self.max_y()
            }
        };

        self.translate(horiz, vert)
    }
    pub fn overlaps(&self, other: &Feature) -> bool {
        for a in self.tiles.iter() {
            for b in other.tiles.iter() {
                if a == b { return true; }
            }
        }

        return false;
    }
    pub fn iter(&self) -> ::std::slice::Iter<Tile> {
        self.tiles.iter()
    }
    pub fn walls<'a>(&'a self) -> Box<Iterator<Item=&'a Location> + 'a> {
        Box::new(self.tiles.iter()
            .filter(|c| c.terrain == Terrain::Wall)
            .map(|c| &c.loc))
    }
    pub fn floors<'a>(&'a self) -> Box<Iterator<Item=&'a Location> + 'a> {
        Box::new(self.tiles.iter()
            .filter(|c| c.terrain == Terrain::Floor)
            .map(|c| &c.loc))
    }
    pub fn min_x(&self) -> i32 {
        self.tiles.iter().map(|t| t.loc.x).min().unwrap()
    }
    pub fn max_x(&self) -> i32 {
        self.tiles.iter().map(|t| t.loc.x).max().unwrap()
    }
    pub fn min_y(&self) -> i32 {
        self.tiles.iter().map(|t| t.loc.y).min().unwrap()
    }
    pub fn max_y(&self) -> i32 {
        self.tiles.iter().map(|t| t.loc.y).max().unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right
}

impl HorizontalAlignment {
    pub fn flip(self) -> Self {
        match self {
            HorizontalAlignment::Left => HorizontalAlignment::Right,
            HorizontalAlignment::Right => HorizontalAlignment::Left,
            HorizontalAlignment::Center => HorizontalAlignment::Center
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom
}

impl VerticalAlignment {
    pub fn flip(self) -> Self {
        match self {
            VerticalAlignment::Top => VerticalAlignment::Bottom,
            VerticalAlignment::Bottom => VerticalAlignment::Top,
            VerticalAlignment::Center => VerticalAlignment::Center
        }
    }
}

macro_rules! feature {
    ( $( ($x:expr, $y:expr) => $t:expr ),* ) => {
        {
            Feature::new(vec![$( Tile::new(Location::new($x, $y), $t), )*])
        }
    };
}

#[test]
fn test_place_feature() {
    // Set up a feature:
    //
    // ....
    // .##.
    // .##.
    // .#..
    // ....
    let feature = feature!(
        (1, 1) => Terrain::Wall,
        (2, 1) => Terrain::Wall,
        (1, 2) => Terrain::Wall,
        (2, 2) => Terrain::Wall,
        (1, 3) => Terrain::Wall
    );

    // Place top left at (2,3).
    //
    // .....
    // .....
    // .....
    // ..##.
    // ..##.
    // ..#..
    // .....
    assert_eq!(
        feature.place(VerticalAlignment::Top, HorizontalAlignment::Left,
            Location::new(2, 3)),
        feature!(
            (2, 3) => Terrain::Wall,
            (3, 3) => Terrain::Wall,
            (2, 4) => Terrain::Wall,
            (3, 4) => Terrain::Wall,
            (2, 5) => Terrain::Wall
        ));

    // Place bottom right at (5,2).
    //
    // ....##.
    // ....##.
    // ....#..
    // .......
    assert_eq!(
        feature.place(VerticalAlignment::Bottom, HorizontalAlignment::Right,
            Location::new(5, 2)),
        feature!(
            (4, 0) => Terrain::Wall,
            (5, 0) => Terrain::Wall,
            (4, 1) => Terrain::Wall,
            (5, 1) => Terrain::Wall,
            (4, 2) => Terrain::Wall
        ));

    // Set up a square feature.
    //
    // ###
    // ###
    // ###
    let square = feature!(
        (0, 0) => Terrain::Wall,
        (1, 0) => Terrain::Wall,
        (2, 0) => Terrain::Wall,
        (0, 1) => Terrain::Wall,
        (1, 1) => Terrain::Wall,
        (2, 1) => Terrain::Wall,
        (0, 2) => Terrain::Wall,
        (1, 2) => Terrain::Wall,
        (2, 2) => Terrain::Wall
    );

    // Place center at (4, 1).
    //
    // ...###.
    // ...###.
    // ...###.
    // .......
    assert_eq!(
        square.place(VerticalAlignment::Center, HorizontalAlignment::Center,
            Location::new(4, 1)),
        feature!(
            (3, 0) => Terrain::Wall,
            (4, 0) => Terrain::Wall,
            (5, 0) => Terrain::Wall,
            (3, 1) => Terrain::Wall,
            (4, 1) => Terrain::Wall,
            (5, 1) => Terrain::Wall,
            (3, 2) => Terrain::Wall,
            (4, 2) => Terrain::Wall,
            (5, 2) => Terrain::Wall
        ));
}
