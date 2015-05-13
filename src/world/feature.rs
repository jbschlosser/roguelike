use tile::{Tile, Terrain, Location};

// GENERATION STUFF.
// A feature in the world, consisting of some arrangement of terrain.
// Features consist of relative coordinates; they can be placed at any
// arbitrary location.
#[derive(Clone, Debug)]
pub struct Feature {
    tiles: Vec<Tile>
}

impl Feature {
    pub fn new(tiles: Vec<Tile>) -> Self {
        Feature { tiles: tiles }
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
}

#[derive(Clone, Copy, Debug)]
pub enum HorizontalAlignment {
    Left,
    Center,
    Right
}

#[derive(Clone, Copy, Debug)]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom
}

// Build features! Take the raw feature shape and translate it
// according to the given alignment and absolute location.
#[derive(Clone, Debug)]
pub struct FeatureBuilder {
    shape: Vec<Tile>,
    location: Location,
    horiz_align: HorizontalAlignment,
    vert_align: VerticalAlignment
}

impl FeatureBuilder {
    pub fn new(shape: Vec<Tile>) -> Self {
        assert!(shape.len() > 0);
        FeatureBuilder {
            shape: shape,
            location: Location::new(0, 0),
            horiz_align: HorizontalAlignment::Left,
            vert_align: VerticalAlignment::Top
        }
    }
    pub fn room(width: i32, height: i32) -> Self {
        let mut shape = Vec::new();
        for x in 0..width {
            for y in 0..height {
                let terrain =
                    if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                        Terrain::Wall
                    } else {
                        Terrain::Floor
                    };
                shape.push(Tile::new(Location::new(x, y), terrain));
            }
        }

        FeatureBuilder::new(shape)
    }
    // TODO: Factor this out somehow; only the distance function
    // changes between circles and diamonds.
    pub fn diamond_room(radius: i32) -> Self {
        let center = Location::new(0, 0);
        let mut shape = Vec::new();
        for i in -radius..radius+1 {
            for j in -radius..radius+1 {
                let loc = Location::new(i, j);
                let dist = loc.manhattan(&center);
                if dist < radius {
                    shape.push(Tile::new(loc, Terrain::Floor));
                } else if dist == radius {
                    shape.push(Tile::new(loc, Terrain::Wall));
                }
            }
        }

        FeatureBuilder::new(shape)
    }
    pub fn circle_room(radius: i32) -> Self {
        let center = Location::new(0, 0);
        let mut shape = Vec::new();
        for i in -radius..radius+1 {
            for j in -radius..radius+1 {
                let loc = Location::new(i, j);
                let dist = loc.euclidean(&center);
                if dist < radius {
                    shape.push(Tile::new(loc, Terrain::Floor));
                } else if dist == radius {
                    shape.push(Tile::new(loc, Terrain::Wall));
                }
            }
        }

        FeatureBuilder::new(shape)
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
                self.location.x - Self::calc_min_x(&self.shape)
            },
            HorizontalAlignment::Center => {
                self.location.x - (Self::calc_min_x(&self.shape) +
                    (Self::calc_max_x(&self.shape) -
                    Self::calc_min_x(&self.shape) + 1) / 2)
            },
            HorizontalAlignment::Right => {
                self.location.x - Self::calc_max_x(&self.shape)
            }
        };
        let vert = match self.vert_align {
            VerticalAlignment::Top => {
                self.location.y - Self::calc_min_y(&self.shape)
            },
            VerticalAlignment::Center => {
                self.location.y - (Self::calc_min_y(&self.shape) +
                    (Self::calc_max_y(&self.shape) -
                    Self::calc_min_y(&self.shape) + 1) / 2)
            },
            VerticalAlignment::Bottom => {
                self.location.y - Self::calc_max_y(&self.shape)
            }
        };

        let tiles = self.shape.iter()
            .map(|s| {
                Tile::new(Location::new(s.loc.x + horiz, s.loc.y + vert), s.terrain)
            })
            .collect();

        Feature::new(tiles)
    }

    fn calc_min_x(shape: &[Tile]) -> i32 {
        shape.iter().map(|t| t.loc.x).min().unwrap()
    }
    fn calc_max_x(shape: &[Tile]) -> i32 {
        shape.iter().map(|t| t.loc.x).max().unwrap()
    }
    fn calc_min_y(shape: &[Tile]) -> i32 {
        shape.iter().map(|t| t.loc.y).min().unwrap()
    }
    fn calc_max_y(shape: &[Tile]) -> i32 {
        shape.iter().map(|t| t.loc.y).max().unwrap()
    }
}

macro_rules! feat_shape {
    ( $( ($x:expr, $y:expr) => $t:expr ),* ) => {
        {
            vec![$( Tile::new(Location::new($x, $y), $t), )*]
        }
    };
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
    let shape = feat_shape!(
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
    assert_eq!(FeatureBuilder::new(shape.clone())
        .vert_align(VerticalAlignment::Top)
        .horiz_align(HorizontalAlignment::Left)
        .location(Location::new(2, 3))
        .build().tiles,
        feat_shape!(
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
    assert_eq!(FeatureBuilder::new(shape)
        .vert_align(VerticalAlignment::Bottom)
        .horiz_align(HorizontalAlignment::Right)
        .location(Location::new(5, 2))
        .build().tiles,
        feat_shape!(
            (4, 0) => Terrain::Wall,
            (5, 0) => Terrain::Wall,
            (4, 1) => Terrain::Wall,
            (5, 1) => Terrain::Wall,
            (4, 2) => Terrain::Wall
        ));

    // Set up a square feature shape.
    //
    // ###
    // ###
    // ###
    let square = feat_shape!(
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
    assert_eq!(FeatureBuilder::new(square)
        .vert_align(VerticalAlignment::Center)
        .horiz_align(HorizontalAlignment::Center)
        .location(Location::new(4, 1))
        .build().tiles,
        feat_shape!(
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
