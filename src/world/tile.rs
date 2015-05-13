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
    pub fn euclidean(&self, other: &Location) -> i32 {
        let (x1, y1, x2, y2) = (self.x as f32, self.y as f32,
            other.x as f32, other.y as f32);
        ((x2 - x1) * (x2 - x1) + (y2 - y1) * (y2 - y1)).sqrt() as i32
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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Tile {
    pub loc: Location,
    pub terrain: Terrain
}

impl Tile {
    pub fn new(loc: Location, terrain: Terrain) -> Self {
        Tile {loc: loc, terrain: terrain}
    }
}

