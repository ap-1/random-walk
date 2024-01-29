use std::time::Duration;

use nannou::{
    prelude::*,
    rand::{self, seq::IteratorRandom, Rng},
};

fn main() {
    nannou::app(model).update(update).run();
}

type Coord = (f32, f32);

struct Model {
    draw: Draw,
    colors: (Rgba8, Rgb8, Rgb8, Rgba8, Rgba8),
    grid_size: i16,
    grid: Vec<Vec<Rgba8>>,
    previous_update: Duration,
    pos_1: Coord,
    next_pos_1: Coord,
    pos_2: Coord,
    next_pos_2: Coord,
}

fn model(app: &App) -> Model {
    app.new_window().view(view).size(750, 750).build().unwrap();

    let default_color = rgba8(47, 47, 47, 255);
    let color_1 = rgb8(235, 65, 55);
    let color_2 = rgb8(0, 155, 240);

    let color_1a = with_alpha(color_1, 15);
    let color_2a = with_alpha(color_2, 15);

    let grid_size = 2;

    let (pos_1, pos_2) = random_positions(grid_size);
    let (next_pos_1, next_pos_2) = in_random_direction(pos_1, pos_2, grid_size);

    let mut grid = vec![vec![default_color; grid_size as usize]; grid_size as usize];
    grid[pos_1.0 as usize - 1][pos_1.1 as usize - 1] = color_1a;
    grid[pos_2.0 as usize - 1][pos_2.1 as usize - 1] = color_2a;

    Model {
        draw: app.draw(),
        colors: (default_color, color_1, color_2, color_1a, color_2a),
        grid_size,
        grid,
        previous_update: app.duration.since_start,
        pos_1,
        pos_2,
        next_pos_1,
        next_pos_2,
    }
}

fn with_alpha(color: Rgb8, alpha: u8) -> Rgba8 {
    let (r, g, b) = color.into_components();

    rgba8(r, g, b, alpha)
}

fn random_positions(grid_size: i16) -> (Coord, Coord) {
    let mut rng = rand::thread_rng();
    let random_nums: Vec<i16> = (0..4).map(|_| rng.gen_range(1..=grid_size)).collect();

    let pos_1 = (random_nums[0] as f32, random_nums[1] as f32);
    let pos_2 = (random_nums[2] as f32, random_nums[3] as f32);

    (pos_1, pos_2)
}

fn in_random_direction(pos_1: Coord, pos_2: Coord, grid_size: i16) -> (Coord, Coord) {
    let mut rng = rand::thread_rng();
    let dirs: Vec<Coord> = vec![
        (-1.0, 1.0),
        (0.0, 1.0),
        (1.0, 1.0),
        (-1.0, 0.0),
        (1.0, 0.0),
        (-1.0, -1.0),
        (0.0, -1.0),
        (1.0, -1.0),
    ];

    let dir_1 = dirs
        .clone()
        .into_iter()
        .filter(|dir| can_move(pos_1, *dir, grid_size))
        .choose(&mut rng)
        .unwrap();

    let dir_2 = dirs
        .into_iter()
        .filter(|dir| can_move(pos_2, *dir, grid_size))
        .choose(&mut rng)
        .unwrap();

    let new_pos_1 = (pos_1.0 + dir_1.0, pos_1.1 + dir_1.1);
    let new_pos_2 = (pos_2.0 + dir_2.0, pos_2.1 + dir_2.1);

    (new_pos_1, new_pos_2)
}

fn generate_random_color() -> (Rgb8, Rgba8) {
    let mut rng = rand::thread_rng();
    let random_nums: Vec<u8> = (0..3).map(|_| rng.gen_range(0..=255)).collect();

    let color = rgb8(random_nums[0], random_nums[1], random_nums[2]);
    let color_a = with_alpha(color, 15);

    (color, color_a)
}

fn interpolate(pos: Coord, next_pos: Coord, t: f32) -> Coord {
    // quadratic ease in/out - acceleration until halfway, then deceleration
    let eased_factor = if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    };

    let (x1, y1) = pos;
    let (x2, y2) = next_pos;

    let interpolated_x = x1 + (x2 - x1) * eased_factor;
    let interpolated_y = y1 + (y2 - y1) * eased_factor;

    (interpolated_x, interpolated_y)
}

fn coord_to_point((i, j): Coord, window: Rect<f32>, grid_size: i16) -> Coord {
    let tile_x = window.w() / (grid_size as f32 + 1.0);
    let tile_y = window.h() / (grid_size as f32 + 1.0);

    let x = tile_x * i;
    let y = tile_y * j;

    let x = map_range(x, 0.0, window.w(), window.left(), window.right());
    let y = map_range(y, 0.0, window.h(), window.bottom(), window.top());

    (x, y)
}

fn to_grid_indices(
    (x1, y1): Coord,
    (x2, y2): Coord,
    grid_size: i16,
) -> (usize, usize, usize, usize, usize) {
    (
        x1 as usize - 1,
        y1 as usize - 1,
        x2 as usize - 1,
        y2 as usize - 1,
        grid_size as usize,
    )
}

fn can_move(pos: Coord, dir: Coord, grid_size: i16) -> bool {
    let (x, y) = (pos.0 + dir.0, pos.1 + dir.1);
    x >= 1.0 && x <= grid_size as f32 && y >= 1.0 && y <= grid_size as f32
}

fn ccw(a: Coord, b: Coord, c: Coord) -> bool {
    (c.1 - a.1) * (b.0 - a.0) > (b.1 - a.1) * (c.0 - a.0)
}

fn intersect(a: Coord, b: Coord, c: Coord, d: Coord) -> bool {
    // Return true if line segments AB and CD intersect
    ccw(a, c, d) != ccw(b, c, d) && ccw(a, b, c) != ccw(a, b, d)
}

fn update(app: &App, model: &mut Model, _update: Update) {
    model.draw.background().color(rgb8(21, 21, 21));

    let (default_color, color_1, color_2, color_1a, color_2a) = model.colors;

    let circle_size = 7.5;
    let window = app.window_rect();
    let grid_size = model.grid_size;

    let pos_1 = model.pos_1;
    let pos_2 = model.pos_2;

    for i in 1..grid_size + 1 {
        for j in 1..grid_size + 1 {
            let (x, y) = coord_to_point((i as f32, j as f32), window, grid_size);

            model
                .draw
                .ellipse()
                .x_y(x, y)
                .radius(circle_size)
                .color(model.grid[i as usize - 1][j as usize - 1]);
        }
    }

    let current_time = app.duration.since_start;
    let dt = current_time.as_millis() - model.previous_update.as_millis();

    if dt > 1000 {
        model.previous_update = current_time;

        let new_pos_1 = model.next_pos_1;
        let new_pos_2 = model.next_pos_2;

        let (x1, y1, x2, y2, _size) = to_grid_indices(new_pos_1, new_pos_2, grid_size);

        model.pos_1 = new_pos_1;
        model.pos_2 = new_pos_2;

        model.grid[x1][y1] = color_1a;
        model.grid[x2][y2] = color_2a;

        if new_pos_1 == new_pos_2 || intersect(pos_1, new_pos_1, pos_2, new_pos_2) {
            model.grid_size += 1;

            let (next_pos_1, next_pos_2) = random_positions(model.grid_size);
            let (x1, y1, x2, y2, size) = to_grid_indices(next_pos_1, next_pos_2, model.grid_size);

            model.pos_1 = next_pos_1;
            model.pos_2 = next_pos_2;

            let (color_1, color_1a) = generate_random_color();
            let (color_2, color_2a) = generate_random_color();

            model.colors = (default_color, color_1, color_2, color_1a, color_2a);

            model.grid = vec![vec![default_color; size]; size];
            model.grid[x1][y1] = color_1a;
            model.grid[x2][y2] = color_2a;
        }

        let (next_pos_1, next_pos_2) = in_random_direction(model.pos_1, model.pos_2, grid_size);

        model.next_pos_1 = next_pos_1;
        model.next_pos_2 = next_pos_2;
    } else {
        let factor = dt as f32 / 1000.0;

        let interpolated_pos_1 = interpolate(pos_1, model.next_pos_1, factor);
        let interpolated_pos_2 = interpolate(pos_2, model.next_pos_2, factor);

        let screen_pos_1 = coord_to_point(interpolated_pos_1, window, grid_size);
        let screen_pos_2 = coord_to_point(interpolated_pos_2, window, grid_size);

        model
            .draw
            .ellipse()
            .x_y(screen_pos_1.0, screen_pos_1.1)
            .radius(circle_size)
            .color(color_1);

        model
            .draw
            .ellipse()
            .x_y(screen_pos_2.0, screen_pos_2.1)
            .radius(circle_size)
            .color(color_2);
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    model.draw.to_frame(app, &frame).unwrap();
}
