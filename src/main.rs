use image::{DynamicImage, RgbaImage};
use image::io::Reader;
use rand::{thread_rng, Rng};
use rand::rngs::ThreadRng;
use clap::Parser;
use std::cmp::{min, max};
use std::env;

const WEB_MODE_VAR: &str = "BATTLEMAPPER_WEB";
const WEB_QUERY_VAR: &str = "QUERY_STRING";
const TILE_SIZE: u32 = 32;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value_t = 48)]
    width: u8,

    #[clap(short, long, default_value_t = 48)]
    height: u8,

    #[clap(short, long, default_value_t = 6)]
    road_count: u8,

    #[clap(short = 'R', long, default_value_t = 2)]
    road_width: u8,

    #[clap(short, long, default_value_t = 6)]
    building_count: u8,

    #[clap(short = 'B', long, default_value_t = 16)]
    building_size: u8
}

#[derive(Copy, Clone, PartialEq)]
enum Orientation {
    Horiz,
    Vert
}

#[derive(PartialEq)]
struct Point {
    x: u32,
    y: u32
}

impl Point {
    fn new(x: u32, y: u32) -> Point {
        Point{x, y}
    }
}

struct Line {
    x: u32,
    y: u32,
    orientation: Orientation,
    length: u32
}

impl Line {
    fn find_point_within(&self, margin: u32, rng: &mut ThreadRng) -> Point {
        let mut point = Point::new(self.x, self.y);
        let distance = rng.gen_range(margin..self.length-margin);
        match self.orientation {
            Orientation::Horiz => { point.x += distance; },
            Orientation::Vert => { point.y += distance; }
        }
        point
    }

    fn point_intersects(&self, point: &Point) -> bool {
        match self.orientation {
            Orientation::Horiz => {
                point.y == self.y && point.x >= self.x && point.x < self.x + self.length
            },
            Orientation::Vert => {
                point.x == self.x && point.y >= self.y && point.y < self.y + self.length
            }
        }
    }

    fn line_intersects(&self, other_line: Line) -> bool {
        (self.orientation == Orientation::Horiz && 
            other_line.orientation == Orientation::Vert && 
            self.y >= other_line.y && 
            self.y < other_line.y + other_line.length && 
            self.x <= other_line.x && 
            self.x + self.length - 1 >= other_line.x) &&
        (self.orientation == Orientation::Vert && 
            other_line.orientation == Orientation::Horiz && 
            self.x >= other_line.x && 
            self.x < other_line.x + other_line.length &&
            self.y <= other_line.y && 
            self.y + self.length - 1 >= other_line.y)
    }

    fn intersection_point_with(&self, other_line: Line) -> (u32, u32) {
        let mut point = (self.x, self.y);
        match self.orientation {
            Orientation::Horiz => { point.0 = other_line.x },
            Orientation::Vert => { point.1 = other_line.y }
        }
        point
    }
}

#[derive(Copy, Clone)]
struct Rectangle {
    x1: u32,
    y1: u32,
    x2: u32,
    y2: u32
}

impl Rectangle {
    fn width(&self) -> u32 {
        self.x2 - self.x1 + 1
    }

    fn height(&self) -> u32 {
        self.y2 - self.y1 + 1
    }

    fn area(&self) -> u32 {
        self.width() * self.height()
    }

    fn perimeter(&self) -> u32 {
        self.width() * 2 + self.height() * 2 - 4
    }

    fn divisible(&self, min_size: u32) -> bool {
        self.width() >= min_size * 2 || self.height() >= min_size * 2
    }

    fn divide_with_lines(&self, line_count: u32, line_margin: u32, rng: &mut ThreadRng) -> Vec<Line> {
        let mut lines: Vec<Line> = Vec::new();
        for _r in 0..line_count {
            let mut line = Line{x: self.x1, y: self.y1, length: 0, orientation: Orientation::Horiz};
            if lines.is_empty() {
                let mut vert: bool = rng.gen();
                if self.width() <= line_margin * 2 {
                    vert = false;
                }
                if self.height() <= line_margin * 2 {
                    vert = true;
                }
                //println!("width {} height {}", self.width(), self.height());
                if vert {
                    // vertical road
                    line.orientation = Orientation::Vert;
                    if self.x1 + line_margin == self.x2 - line_margin {
                        line.x = self.x1;
                    } else {
                        line.x = rng.gen_range(self.x1 + line_margin..self.x2 - line_margin);
                    }
                    line.length = self.height();
                } else {
                    // horizontal road
                    line.orientation = Orientation::Horiz;
                    if self.y1 + line_margin == self.y2 - line_margin {
                        line.y = self.y1;
                    } else {
                        line.y = rng.gen_range(self.y1 + line_margin..self.y2 - line_margin);
                    }
                    line.length = self.width();
                }
            } else {
                let mut origin_road_num = 0;
                let mut retrying = true;
                while retrying {
                    retrying = false;
                    origin_road_num = rng.gen_range(0..lines.len());
                    if lines[origin_road_num].length > line_margin * 2 {
                        line.orientation = opposite_orientation(lines[origin_road_num].orientation);
                        let intersection = lines[origin_road_num].find_point_within(line_margin, rng);
                        //println!("rect {} {} {} {} origin road {} {} {}", self.x1, self.y1, self.x2, self.y2, lines[origin_road_num].x, lines[origin_road_num].y, lines[origin_road_num].length);
                        let mut low_bound = self.x1;
                        let mut high_bound = self.x2;
                        if line.orientation == Orientation::Vert {
                            low_bound = self.y1;
                            high_bound = self.y2;
                        }
                        //println!("intersection {} {} low bound {}", intersection.x, intersection.y, low_bound);
                        for other_road_num in 0..lines.len() {
                            if other_road_num != origin_road_num {
                                if lines[other_road_num].point_intersects(&intersection) {
                                    retrying = true;
                                } else if lines[other_road_num].orientation != line.orientation {
                                    match line.orientation {
                                        Orientation::Horiz => {
                                            //println!("horiz {} {}", line.y, line.length);
                                            if lines[other_road_num].y <= intersection.y && lines[other_road_num].y + lines[other_road_num].length - 1 >= intersection.y {
                                                if lines[other_road_num].x < intersection.x && lines[other_road_num].x > low_bound {
                                                    low_bound = lines[other_road_num].x;
                                                    //println!("set low bound to {}", low_bound);
                                                } else if lines[other_road_num].x > intersection.x && lines[other_road_num].x < high_bound {
                                                    high_bound = lines[other_road_num].x;
                                                }
                                            }
                                        },
                                        Orientation::Vert => {
                                            //println!("vert {} {}", line.x, line.length);
                                            if lines[other_road_num].x <= intersection.x && lines[other_road_num].x + lines[other_road_num].length - 1 >= intersection.x {
                                                if lines[other_road_num].y < intersection.y && lines[other_road_num].y > low_bound {
                                                    low_bound = lines[other_road_num].y;
                                                    //println!("set low bound to {}", low_bound);
                                                } else if lines[other_road_num].y > intersection.y && lines[other_road_num].y < high_bound {
                                                    high_bound = lines[other_road_num].y;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if !retrying {
                            let mut before = true;
                            match line.orientation {
                                Orientation::Horiz => {
                                    //println!("horiz x {} low {} high {}", intersection.x, low_bound, high_bound);
                                    if intersection.x - low_bound < line_margin {
                                        before = false;
                                    } else if high_bound - intersection.x < line_margin {
                                        before = true;
                                    } else {
                                        before = rng.gen();
                                    }
                                    line.y = intersection.y;
                                    if before {
                                        line.x = low_bound;
                                        line.length = intersection.x - low_bound;
                                    } else {
                                        line.x = lines[origin_road_num].x;
                                        line.length = high_bound - intersection.x + 1;
                                    }
                                },
                                Orientation::Vert => {
                                    //println!("vert y {} low {} high {}", intersection.y, low_bound, high_bound);
                                    if intersection.y - low_bound < line_margin {
                                        before = false;
                                    } else if high_bound - intersection.y < line_margin {
                                        before = true;
                                    } else {
                                        before = rng.gen();
                                    }
                                    line.x = intersection.x;
                                    if before {
                                        line.y = low_bound;
                                        line.length = intersection.y - low_bound;
                                    } else {
                                        line.y = lines[origin_road_num].y;
                                        line.length = high_bound - intersection.y + 1;
                                    }
                                }
                            };
                        }
                    }
                }
            }
            //println!("new line {} {} {}", line.x, line.y, line.length);
            lines.push(line);
        }
    
        lines
    }

    fn randomly_divide(&self, min_size: u32, rng: &mut ThreadRng) -> (Rectangle, Rectangle) {
        let mut rect1 = *self;
        let mut rect2 = *self;
        let mut division_line = Orientation::Horiz;
        let x_range = (self.x2 - self.x1) as i32 - min_size as i32 * 2;
        let y_range = (self.y2 - self.y1) as i32 - min_size as i32 * 2;
        if x_range >= 0 && y_range >= 0 {
            if rng.gen::<bool>() {
                division_line = Orientation::Vert;
            }
        } else if x_range >= 0 {
            division_line = Orientation::Vert;
        }
        match division_line {
            Orientation::Horiz => {
                let mut top_size = min_size;
                if y_range > 0 {
                    top_size = rng.gen_range(0..y_range as u32) + min_size;
                }
                rect1.y2 = rect1.y1 + top_size - 1;
                rect2.y1 = rect1.y1 + top_size;
            },
            Orientation::Vert => {
                let mut left_size = min_size;
                if x_range > 0 {
                    left_size = rng.gen_range(0..x_range as u32) + min_size;
                }
                rect1.x2 = rect1.x1 + left_size - 1;
                rect2.x1 = rect1.x1 + left_size;
            }
        }
        (rect1, rect2)
    }

    fn intersection_with(&self, other_rect: Rectangle) -> Rectangle {
        //println!("intersection {} {} {} {} / {} {} {} {}", self.x1, self.y1, self.x2, self.y2, other_rect.x1, other_rect.y1, other_rect.x2, other_rect.y2);
        let mut intersection = *self;
        if other_rect.x1 > intersection.x1 {
            intersection.x1 = other_rect.x1;
        }
        if other_rect.x2 < intersection.x2 {
            intersection.x2 = other_rect.x2;
        }
        if other_rect.y1 > intersection.y1 {
            intersection.y1 = other_rect.y1;
        }
        if other_rect.y2 < intersection.y2 {
            intersection.y2 = other_rect.y2;
        }
        intersection
    }

    fn connecting_border_with(&self, other_rect: Rectangle) -> Line {
        let mut border = Line {x: 0, y: 0, orientation: Orientation::Horiz, length: 0};
        if self.y2 < other_rect.y1 {
            border.x = max(self.x1, other_rect.x1);
            border.y = self.y2;
            border.length = min(self.x2, other_rect.x2) - border.x;
        } else if self.y1 > other_rect.y2 {
            border.x = max(self.x1, other_rect.x1);
            border.y = self.y1;
            border.length = min(self.x2, other_rect.x2) - border.x;
        } else if self.x2 < other_rect.x1 {
            border.y = max(self.y1, other_rect.y1);
            border.x = self.x2;
            border.length = min(self.y2, other_rect.y2) - border.y;
            border.orientation = Orientation::Vert;
        } else {
            border.y = max(self.y1, other_rect.y1);
            border.x = self.x1;
            border.length = min(self.y2, other_rect.y2) - border.y;
            border.orientation = Orientation::Vert;
        }
        border
    }

    fn find_point_within(&self, margin: u32, rng: &mut ThreadRng) -> Point {
        Point::new(rng.gen_range(self.x1+margin..self.x2-margin+1), rng.gen_range(self.y1+margin..self.y2-margin+1))
    }

    fn find_exterior_point(&self, rng: &mut ThreadRng) -> Point {
        let mut point = Point::new(self.x1, self.y1);
        let horiz_wall: bool = rng.gen();
        let lowest: bool = rng.gen();
        if horiz_wall {
            point.x = rng.gen_range(self.x1+1..self.x2);
            if !lowest {
                point.y = self.y2;
            }
        } else {
            point.y = rng.gen_range(self.y1+1..self.y2);
            if !lowest {
                point.x = self.x2;
            }
        }
        point
    }

    fn shrink(&mut self, amount: u32) {
        self.x1 += amount;
        self.y1 += amount;
        self.x2 -= amount;
        self.y2 -= amount;
    }
}

struct Obstructions {
    w: u32,
    h: u32,
    tiles: Vec<bool>,
    unobstructed_count: u32
}

impl Obstructions {
    fn new(width: u32, height: u32) -> Obstructions {
        let mut t: Vec<bool> = Vec::new();
        t.resize((width * height) as usize, false);
        Obstructions {w: width, h: height, tiles: t, unobstructed_count: 0}
    }

    fn obstruct(&mut self, x: u32, y: u32, obstructed: bool) {
        let t = (y * self.w + x) as usize;
        if obstructed && !self.tiles[t] {
            self.unobstructed_count += 1;
        } else if !obstructed && self.tiles[t] {
            self.unobstructed_count -= 1;
        }
        self.tiles[t] = obstructed;
    }

    fn is_obstructed(&self, x: u32, y: u32) -> bool {
        self.tiles[(y * self.w + x) as usize]
    }

    fn obstructed_rectangle(&self, r: &Rectangle) -> bool {
        //println!("obstructed_rectangle {} {} {} {}", r.x1, r.y1, r.x2, r.y2);
        let mut obstructed = false;
        for x in r.x1..r.x2+1 {
            for y in r.y1..r.y2+1 {
                if self.is_obstructed(x, y) {
                    obstructed = true
                }
            }
        }
        obstructed
    }

    fn get_unobstructed_count(&self) -> u32 {
        self.unobstructed_count
    }

    fn find_clear_tile(&self, rng: &mut ThreadRng) -> (u32, u32) {
        let mut choosing = true;
        let mut x = 0;
        let mut y = 0;
        while choosing {
            choosing = false;
            x = rng.gen_range(0..self.w);
            y = rng.gen_range(0..self.h);
            if self.is_obstructed(x, y) {
                choosing = true;
            }
        }
        (x, y)
    }

    fn find_clear_rectangle(&self, min_size: u32, max_size: u32, rng: &mut ThreadRng) -> Rectangle {
        let outer_bounds = Rectangle{x1: 0, y1: 0, x2: self.w - 1, y2: self.h -1};
        //let mut rectangle = outer_bounds;
        let mut ok = false;
        let mut size_x = min_size;
        let mut size_y = min_size;
        let mut point = Point::new(0, 0);

        while !ok {
            ok = true;
    
            point = outer_bounds.find_point_within(min_size + 1, rng);
            //println!("rect {} {} {} {} point {} {}", rectangle.x1, rectangle.y1, rectangle.x2, rectangle.y2, point.0, point.1);
            if self.obstructed_rectangle(&Rectangle {x1: point.x - size_x - 1, y1: point.y - size_y - 1, x2: point.x + size_x + 1, y2: point.y + size_y + 1 }) {
                ok = false;
            } else {
                let mut growing_x = true;
                let mut growing_y = true;
                while growing_x || growing_y {
                    //println!("looping");
                    if growing_x {
                        if point.x > size_x + 2 && point.x + size_x < self.w - 3 && size_x < max_size / 2 {
                            size_x += 1;
                            if self.obstructed_rectangle(&Rectangle {x1: point.x - size_x - 1, y1: point.y - size_y - 1, x2: point.x + size_x + 1, y2: point.y + size_y + 1 }) {
                                growing_x = false;
                            }    
                        } else {
                            growing_x = false;
                        }
                    }
                    if growing_y {
                        if point.y > size_y + 2 && point.y + size_y < self.h - 3 && size_y < max_size / 2 {
                            size_y += 1;
                            if self.obstructed_rectangle(&Rectangle {x1: point.x - size_x - 1, y1: point.y - size_y - 1, x2: point.x + size_x + 1, y2: point.y + size_y + 1 }) {
                                growing_y = false;
                            }    
                        } else {
                            growing_y = false;
                        }
                    }
                }
            }
        }
        //println!("clear rectangle point {} {} size {} {}", point.x, point.y, size_x, size_y);
        Rectangle {x1: point.x - size_x, y1: point.y - size_y, x2: point.x + size_x, y2: point.y + size_y }
    }    
}

fn opposite_orientation(original: Orientation) -> Orientation {
    match original {
        Orientation::Vert => Orientation::Horiz,
        Orientation::Horiz => Orientation::Vert
    }
}

struct BattleMap {
    w: u32,
    h: u32,
    road_count: u32,
    road_width: u32,
    building_count: u32,
    building_size: u32,
    img: RgbaImage
}

impl BattleMap {
    fn new(w: u32, h: u32, road_count: u32, road_width: u32, building_count: u32, building_size: u32 ) -> BattleMap {        
        BattleMap {
            w, 
            h, 
            road_count, 
            road_width, 
            building_count, 
            building_size,
            img: RgbaImage::new(w * TILE_SIZE, h * TILE_SIZE)
        }
    }

    fn pixel_dimensions(&self) -> (u32, u32) {
        (self.w * TILE_SIZE, self.h * TILE_SIZE)
    }

    fn road_margin(&self) -> u32 {
        self.road_width / 2 + 1
    }

    fn generate(&mut self) {
        let mut bytes: Vec<u8> = Vec::new();
        let mut rng = thread_rng();
        let dim = self.pixel_dimensions();
        self.img = RgbaImage::new(dim.0, dim.1);

        let raw_tiles = match Reader::open("gfx/tiles.png") {
            Ok(raw_tiles) => raw_tiles,
            Err(_e) => return
        };
        let mut tiles = match raw_tiles.decode() {
            Ok(tiles) => tiles,
            Err(_e) => return
        };
    
        let mut obstructions = Obstructions::new(self.w, self.h);
    
        // dirt
    
        let dirt_tile = image::imageops::crop(&mut tiles, 0, 0, TILE_SIZE, TILE_SIZE);
    
        for x in 0..self.w {
            for y in 0..self.h {
                image::imageops::overlay(&mut self.img, &dirt_tile, x * TILE_SIZE, y * TILE_SIZE);
            }
        }
    
        // roads
    
        let full_rect = Rectangle{ x1: 0, y1: 0, x2: self.w - 1, y2: self.h - 1 };
        let roads = full_rect.divide_with_lines(self.road_count, self.road_margin(), &mut rng);
    
        let dirt_tile = tiles.crop_imm(32, 0, TILE_SIZE, TILE_SIZE);
        let car_h_tile = tiles.crop_imm(64, 32, 64, 32);
        let car_v_tile = tiles.crop_imm(128, 0, 32, 64);
    
        for road in &roads {
            let mut x = road.x;
            let mut y = road.y;
            //println!("road origin {} {} {}", x, y, road.length);
            for _t in 0..road.length {
                obstructions.obstruct(x, y, true);
                match road.orientation {
                    Orientation::Horiz => {
                        for w in 0..self.road_width {
                            //println!("overlay {} {} {}", x, y, w);
                            image::imageops::overlay(&mut self.img, &dirt_tile, x * TILE_SIZE, (y - (self.road_width / 2) + w) * TILE_SIZE);
                        }
                        for w in 0..self.road_margin() {
                            obstructions.obstruct(x, y - w, true);
                            obstructions.obstruct(x, y + w, true);
                        }
                        x += 1;
                    },
                    Orientation::Vert => {
                        for w in 0..self.road_width {
                            image::imageops::overlay(&mut self.img, &dirt_tile, (x - (self.road_width / 2) + w) * TILE_SIZE, y * TILE_SIZE);
                        }
                        for w in 0..self.road_margin() {
                            obstructions.obstruct(x - w, y, true);
                            obstructions.obstruct(x + w, y, true);
                        }
                        y += 1;
                    }
                }
            }
        }
    
        for road in &roads {
            if road.length > 4 {
                let car = road.find_point_within(2, &mut rng);
                if rng.gen::<bool>() {
                    image::imageops::overlay(&mut self.img, &car_v_tile, car.x * TILE_SIZE, car.y * TILE_SIZE);    
                } else {
                    image::imageops::overlay(&mut self.img, &car_h_tile, car.x * TILE_SIZE, car.y * TILE_SIZE);    
                }
                
            }
        }
    
        // buildings
    
        //println!("start buildings");
    
        let floor_tile = tiles.crop_imm(96, 0, TILE_SIZE, TILE_SIZE);
        let wall_nw_tile = tiles.crop_imm(0, 96, TILE_SIZE, TILE_SIZE);
        let wall_ne_tile = tiles.crop_imm(32, 96, TILE_SIZE, TILE_SIZE);
        let wall_sw_tile = tiles.crop_imm(64, 96, TILE_SIZE, TILE_SIZE);
        let wall_se_tile = tiles.crop_imm(96, 96, TILE_SIZE, TILE_SIZE);
        let wall_n_tile = tiles.crop_imm(128, 96, TILE_SIZE, TILE_SIZE);
        let wall_s_tile = tiles.crop_imm(160, 96, TILE_SIZE, TILE_SIZE);
        let wall_w_tile = tiles.crop_imm(192, 96, TILE_SIZE, TILE_SIZE);
        let wall_e_tile = tiles.crop_imm(224, 96, TILE_SIZE, TILE_SIZE);
        let door_w_tile = tiles.crop_imm(0, 64, TILE_SIZE, TILE_SIZE);
        let door_n_tile = tiles.crop_imm(32, 64, TILE_SIZE, TILE_SIZE);
        let door_e_tile = tiles.crop_imm(64, 64, TILE_SIZE, TILE_SIZE);
        let door_s_tile = tiles.crop_imm(96, 64, TILE_SIZE, TILE_SIZE);
        let crate_tile = tiles.crop_imm(0, 32, TILE_SIZE, TILE_SIZE);
    
        for b in 0..self.building_count {
            //println!("building {}", b);
            let mut building = obstructions.find_clear_rectangle(3, self.building_size, &mut rng);
            building.shrink(1);
            let door_count = building.perimeter() / 20 + 1;
            let mut doors = Vec::new();
            for _d in 0..door_count {
                doors.push(building.find_exterior_point(&mut rng));
            }
            for x in building.x1..building.x2+1 {
                for y in building.y1..building.y2+1 {
                    image::imageops::overlay(&mut self.img, &floor_tile, x * TILE_SIZE, y * TILE_SIZE);
                    let point = Point::new(x, y);
                    if doors.contains(&point) {
                        if x == building.x1 {
                            image::imageops::overlay(&mut self.img, &door_w_tile, x * TILE_SIZE, y * TILE_SIZE);
                        } else if x == building.x2 {
                            image::imageops::overlay(&mut self.img, &door_e_tile, x * TILE_SIZE, y * TILE_SIZE);
                        } else if y == building.y1 {
                            image::imageops::overlay(&mut self.img, &door_n_tile, x * TILE_SIZE, y * TILE_SIZE);
                        } else {
                            image::imageops::overlay(&mut self.img, &door_s_tile, x * TILE_SIZE, y * TILE_SIZE);
                        }
                    } else if x == building.x1 {
                        if y == building.y1 {
                            image::imageops::overlay(&mut self.img, &wall_nw_tile, x * TILE_SIZE, y * TILE_SIZE);
                        } else if y == building.y2 {
                            image::imageops::overlay(&mut self.img, &wall_sw_tile, x * TILE_SIZE, y * TILE_SIZE);
                        } else {
                            image::imageops::overlay(&mut self.img, &wall_w_tile, x * TILE_SIZE, y * TILE_SIZE);
                        }
                    } else if x == building.x2 {
                        if y == building.y1 {
                            image::imageops::overlay(&mut self.img, &wall_ne_tile, x * TILE_SIZE, y * TILE_SIZE);
                        } else if y == building.y2 {
                            image::imageops::overlay(&mut self.img, &wall_se_tile, x * TILE_SIZE, y * TILE_SIZE);
                        } else {
                            image::imageops::overlay(&mut self.img, &wall_e_tile, x * TILE_SIZE, y * TILE_SIZE);
                        }
                    } else if y == building.y1 {
                        image::imageops::overlay(&mut self.img, &wall_n_tile, x * TILE_SIZE, y * TILE_SIZE);
                    } else if y == building.y2 {
                        image::imageops::overlay(&mut self.img, &wall_s_tile, x * TILE_SIZE, y * TILE_SIZE);
                    }    
                    obstructions.obstruct(x, y, true);
                }
            }
    
    
            //let mut walls = Vec::new();
            let wall_count = building.area() / 30;
            let walls = building.divide_with_lines(wall_count, 3, &mut rng);
            for wall in &walls {
                let mut draw_length = wall.length;
                match wall.orientation {
                    Orientation::Horiz => {
                        if wall.x > building.x1 && wall.x + wall.length <= building.x2 {
                            draw_length -= 1;
                        }
                    },
                    Orientation::Vert => {
                        if wall.y > building.y1 && wall.y + wall.length <= building.y2 {
                            draw_length -= 1;
                        }
                    }
                }
                let mut door = 1;
                if draw_length > 3 {
                    door = rng.gen_range(1..draw_length-2);
                }
                for l in 0..draw_length {
                    if l == door {
                        match wall.orientation {
                            Orientation::Horiz => {
                                image::imageops::overlay(&mut self.img, &door_n_tile, (wall.x + l) * TILE_SIZE, wall.y * TILE_SIZE);
                            },
                            Orientation::Vert => {
                                image::imageops::overlay(&mut self.img, &door_w_tile, wall.x * TILE_SIZE, (wall.y + l) * TILE_SIZE);
                            }
                        }
                    } else {
                        match wall.orientation {
                            Orientation::Horiz => {
                                image::imageops::overlay(&mut self.img, &wall_n_tile, (wall.x + l) * TILE_SIZE, wall.y * TILE_SIZE);
                            },
                            Orientation::Vert => {
                                image::imageops::overlay(&mut self.img, &wall_w_tile, wall.x * TILE_SIZE, (wall.y + l) * TILE_SIZE);
                            }
                        }    
                    }
                }
            }
            
            let obstacles = building.area() / 50;
            for _o in 0..obstacles {
                let mut thing = Point::new(0, 0);
                let mut finding = true;
                while finding {
                    finding = false;
                    thing = building.find_point_within(1, &mut rng);
                    for wall in &walls {
                        if wall.point_intersects(&thing) {
                            finding = true;
                        }
                    }
                }
                image::imageops::overlay(&mut self.img, &crate_tile, thing.x * TILE_SIZE, thing.y * TILE_SIZE);
            }
        }
    
        // outdoor obstacles
    
        //println!("start obstacles");
    
        let bush_tile = tiles.crop_imm(32, 32, TILE_SIZE, TILE_SIZE);
        let obstacles = obstructions.get_unobstructed_count() / 50;
        for _o in 0..obstacles {
            let coords = obstructions.find_clear_tile(&mut rng);
            obstructions.obstruct(coords.0, coords.1, true);
            image::imageops::overlay(&mut self.img, &bush_tile, coords.0 * TILE_SIZE, coords.1 * TILE_SIZE);
        }
    
        // grid
    
        for x in 0..self.img.width() {
            for y in 0..self.img.height() {
                if x % TILE_SIZE == 0 || y % TILE_SIZE == 0 {
                    let pixel = self.img.get_pixel_mut(x, y);
                    //let image::Rgba(data) = *pixel;
                    *pixel = image::Rgba([128, 128, 128, 255]);
                }
            }
        }
    
        // done, save
    
        /*
        match self.img.save("map.png") {
            Ok(_ok) => (),
            Err(_err) => ()
        } 
        */
    }

    fn base64(&self)-> String {
        let mut bytes: Vec<u8> = Vec::new();
        let temp_img = RgbaImage::from_vec(self.w * TILE_SIZE, self.h * TILE_SIZE, self.img.as_raw().to_vec());
        match temp_img {
            Some(img) => {
                let dyn_img = DynamicImage::ImageRgba8(img);
                dyn_img.write_to(&mut bytes, image::ImageOutputFormat::Png);        
            },
            None => {}
        }
        base64::encode(bytes)
    }

    fn save_to(&self, filename: &str) {
        self.img.save(filename);
    }
}

// main program function

fn main() {
    //println!("Apocalypsing...");

    let mut args = Args::parse();

    let mut web_mode = false;
    match env::var(WEB_MODE_VAR) {
        Ok(val) => { web_mode = val.eq("1") },
        Err(_err) => {}
    }

    if web_mode {
        match env::var(WEB_QUERY_VAR) {
            Ok(val) => { 
                let mut env_args: Vec<&str> = val.split('&').collect();
                env_args.insert(0, "");
                eprintln!("{:?}", env_args);
                args = Args::parse_from(env_args);
            },
            Err(_err) => {}
        }
    }

    eprintln!("args {:?}", args);

    let mut map = BattleMap::new(
        args.width as u32,
        args.height as u32,
        args.road_count as u32,
        args.road_width as u32,
        args.building_count as u32,
        args.building_size as u32
    );

    eprintln!("{} {}", map.w, map.h);

    map.generate();

    if web_mode {
        let img_b64 = map.base64();
        println!("Content-type: text/plain\n");
        println!("{}", img_b64);    
    } else {
        map.save_to("map.png");
    }

    //println!("<html><body><p>Hello world</p><img src=\"data:image/png;base64,{}\"></body></html>", img_b64);
}

