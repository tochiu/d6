use crate::transform::*;

use tui::style::Color;

use std::{marker::PhantomData, ops::RangeInclusive};

const SCENE_WORLD_UNITS_PER_PIXEL: f64 = 1.0;

pub type BoxedIterator<'a, T> = Box<dyn Iterator<Item = T> + 'a>;

pub trait Scene {
    fn camera_transform(&self) -> &Transform;
    fn update_geometry(&self, geometry_buffer: &mut Vec<SceneTriangle>);
}

// TODO: Decouple color from triangle in scene (not even sure what the best way to architect computing color)
pub struct SceneTriangle {
    pub points: [Vector3; 3],
    pub normal: Vector3,
    pub color: Color,
}

pub trait Viewport: Transformable {
    fn new(transform: Transform, width: f64, height: f64) -> Self;
    fn project_geometry<'a>(
        &'a self,
        geometry: &'a [SceneTriangle],
    ) -> BoxedIterator<'a, (&'a SceneTriangle, TriangleProjection)>;
}

#[derive(Clone)]
pub struct OrthographicCamera {
    pub transform: Transform,
    pub width: f64,
    pub height: f64,
}

impl OrthographicCamera {
    fn point_to_projection_space(&self, mut point: Vector3) -> Vector3 {
        point = self.transform.point_to_local_space(point);
        point.x = (point.x + 0.5 * self.width) / self.width;
        point.y = 1.0 - (point.y + 0.5 * self.height) / self.height;
        point
    }

    fn are_projection_bounds_within_viewport_bounds(&self, projection_points: &[Vector3]) -> bool {
        let x0 = projection_points
            .iter()
            .fold(f64::MAX, |x0, point| x0.min(point.x));
        let y0 = projection_points
            .iter()
            .fold(f64::MAX, |y0, point| y0.min(point.y));
        let x1 = projection_points
            .iter()
            .fold(f64::MIN, |x1, point| x1.max(point.x));
        let y1 = projection_points
            .iter()
            .fold(f64::MIN, |y1, point| y1.max(point.y));

        x0 < 1.0 && x1 > 0.0 && y0 < 1.0 && y1 > 0.0
    }
}

impl Viewport for OrthographicCamera {
    fn new(transform: Transform, width: f64, height: f64) -> Self {
        Self {
            transform,
            width,
            height,
        }
    }

    fn project_geometry<'a>(
        &'a self,
        geometry: &'a [SceneTriangle],
    ) -> BoxedIterator<'a, (&'a SceneTriangle, TriangleProjection)> {
        //log::info!("{:?}", self.transform);
        let (camera_right, camera_up, camera_look) = self.transform.rotation.basis_vectors();
        Box::new(geometry.iter().filter_map(move |tri: &SceneTriangle| {
            // Backface culling
            if tri.normal.dot(camera_look) >= 0.0 {
                return None;
            }

            let mut projection_points = tri.points;
            for point in projection_points.iter_mut() {
                *point = self.point_to_projection_space(*point);
            }

            // Cull triangles that are completely off screen
            if !self.are_projection_bounds_within_viewport_bounds(&projection_points) {
                return None;
            }

            // Calculate the distance the plane containing the triangle recedes from the camera plane
            // when traversing the camera's width and height
            let projection_distance_change = Vector3 {
                x: -tri.normal.dot(self.width * camera_right) / tri.normal.dot(camera_look),
                y: tri.normal.dot(self.height * camera_up) / tri.normal.dot(camera_look),
                z: 0.0,
            };

            Some((
                tri,
                TriangleProjection {
                    projection_points,
                    projection_distance_change,
                },
            ))
        }))
    }
}

impl Transformable for OrthographicCamera {
    fn transform(&self) -> &Transform {
        &self.transform
    }

    fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

#[derive(Debug)]
pub struct TriangleProjection {
    pub projection_points: [Vector3; 3],
    pub projection_distance_change: Vector3,
}

pub struct Raster {
    pub z_buffer: Buffer2D<f64>,
    pub screen_buffer: Buffer2D<Color>,
    pub geometry_buffer: Vec<SceneTriangle>,
    pub horizonal_line_buffer: Vec<(i32, i32)>,
}

impl Raster {
    fn rasterize(
        &mut self,
        scene: &impl Scene,
        camera: impl Viewport,
        screen_width: usize,
        screen_height: usize,
    ) {
        self.z_buffer
            .clear_and_resize(screen_width, screen_height, f64::INFINITY);

        self.screen_buffer
            .clear_and_resize(screen_width, screen_height, Color::Rgb(0, 0, 0));

        self.horizonal_line_buffer
            .resize(screen_height, (i32::MAX, i32::MAX));

        scene.update_geometry(&mut self.geometry_buffer);

        for (scene_tri, tri_proj) in camera.project_geometry(&self.geometry_buffer) {
            let mut tri_screen_points = [(0, 0); 3];

            for (projection_point, screen_point) in tri_proj
                .projection_points
                .iter()
                .zip(tri_screen_points.iter_mut())
            {
                *screen_point = (
                    (projection_point.x * screen_width as f64).round() as i32,
                    (projection_point.y * screen_height as f64).round() as i32,
                );
            }

            for line in [
                [&tri_screen_points[0], &tri_screen_points[1]],
                [&tri_screen_points[1], &tri_screen_points[2]],
                [&tri_screen_points[2], &tri_screen_points[0]],
            ] {
                let &(x1, y1) = line[0];
                let &(x2, y2) = line[1];

                let m = (y2 - y1) as f64 / (x2 - x1) as f64;

                for y in y1.min(y2).max(0)..=y1.max(y2).min(screen_height as i32 - 1) {
                    let x = ((y - y1) as f64 / m).round() as i32 + x1;
                    let row = &mut self.horizonal_line_buffer[y as usize];
                    if row.0 == i32::MAX {
                        *row = (x, x)
                    } else if x < row.0 {
                        row.0 = x
                    } else if x > row.1 {
                        row.1 = x
                    }
                }

                // // if dx > dy {
                // //     self.bressenham_line(x1, y1, x2, y2, dx, dy, 0);
                // // } else {
                // //     self.bressenham_line(y1, x1, y2, x2, dy, dx, 1);
                // // }
            }

            let TriangleProjection {
                projection_points:
                    [Vector3 {
                        x: x0,
                        y: y0,
                        z: z0,
                    }, ..],
                projection_distance_change:
                    Vector3 {
                        x: z_dx, y: z_dy, ..
                    },
            } = tri_proj;

            let y_min = tri_screen_points
                .iter()
                .fold(i32::MAX, |min, &(_, y)| min.min(y))
                .clamp(0, screen_height.saturating_sub(1) as i32) as usize;
            let y_max = tri_screen_points
                .iter()
                .fold(i32::MIN, |max, &(_, y)| max.max(y))
                .clamp(0, screen_height.saturating_sub(1) as i32) as usize;

            for y in y_min..=y_max {
                let (x_min, x_max) = {
                    let (min, max) = self.horizonal_line_buffer[y];
                    if min < 0 && max < 0 || min > screen_width as i32 && max > screen_width as i32
                    {
                        continue;
                    }
                    (
                        min.clamp(0, screen_width as i32 - 1) as usize,
                        max.clamp(0, screen_width as i32 - 1) as usize,
                    )
                };

                let zy = z0 + ((y as f64 + 0.5) / (screen_height as f64) - y0) * z_dy;

                let z_slice = self.z_buffer.get_range_mut(y, x_min..=x_max);
                let pixel_slice = self.screen_buffer.get_range_mut(y, x_min..=x_max);

                for (i, (last_z, pixel)) in
                    z_slice.iter_mut().zip(pixel_slice.iter_mut()).enumerate()
                {
                    let z = zy + (((x_min + i) as f64 + 0.5) / (screen_width as f64) - x0) * z_dx;
                    if z > *last_z {
                        continue;
                    }

                    *last_z = z;
                    *pixel = scene_tri.color;
                }
            }

            self.horizonal_line_buffer[y_min..=y_max].fill((i32::MAX, i32::MAX));
        }
    }

    // fn bressenham_line(
    //     &mut self,
    //     mut x1: i32,
    //     mut y1: i32,
    //     x2: i32,
    //     y2: i32,
    //     dx: i32,
    //     dy: i32,
    //     decide: i32,
    // ) {
    //     let mut pk = 2 * dy - dx;

    //     for _ in 0..dx {
    //         if x1 < x2 {
    //             x1 += 1;
    //         } else {
    //             x1 -= 1;
    //         }

    //         if pk < 0 {
    //             let (x1, y1) = if decide == 0 { (x1, y1) } else { (y1, x1) };
    //             if y1 >= 0 && y1 < self.screen_buffer.height as i32 {
    //                 self.insert_bressenham_point(x1, y1 as usize);
    //             }
    //             pk += 2 * dy;
    //         } else {
    //             if y1 < y2 {
    //                 y1 += 1;
    //             } else {
    //                 y1 -= 1;
    //             }
    //             let (x1, y1) = if decide == 0 { (x1, y1) } else { (y1, x1) };
    //             if y1 >= 0 && y1 < self.screen_buffer.height as i32 {
    //                 self.insert_bressenham_point(x1, y1 as usize);
    //             }
    //             pk += 2 * dy - 2 * dx;
    //         }
    //     }
    // }

    // fn insert_bressenham_point(&mut self, x1: i32, y1: usize) {
    //     let row = &mut self.horizonal_line_buffer[y1];
    //     if row.0 == i32::MAX {
    //         *row = (x1, x1)
    //     } else if x1 < row.0 {
    //         row.0 = x1
    //     } else if x1 > row.1 {
    //         row.1 = x1
    //     }
    // }
}

impl Default for Raster {
    fn default() -> Self {
        Raster {
            z_buffer: Buffer2D::default(),
            screen_buffer: Buffer2D {
                data: vec![Color::Black; 0],
                width: 0,
                height: 0,
            },
            geometry_buffer: Default::default(),
            horizonal_line_buffer: Vec::new(),
        }
    }
}

pub struct RasterWidget<'a, S: Scene, V: Viewport> {
    pub aliasing: usize,
    pub raster: &'a mut Raster,
    pub scene: &'a S,
    pub camera: PhantomData<V>,
}

impl<'a, S: Scene, V: Viewport> RasterWidget<'a, S, V> {
    pub fn new(raster: &'a mut Raster, scene: &'a S, aliasing: usize) -> Self {
        Self {
            aliasing,
            raster,
            scene,
            camera: PhantomData,
        }
    }

    fn sample_color(&self, x: usize, y: usize) -> Color {
        let sample_size = self.aliasing;
        let (sr, sg, sb) = self
            .raster
            .screen_buffer
            .area(sample_size * x, sample_size * y, sample_size, sample_size)
            .flatten()
            .fold((0_u16, 0_u16, 0_u16), |(sr, sg, sb), color| {
                let Color::Rgb(r, g, b) = *color else {
                    unreachable!("Color {:?} is not RGB", color)
                };
                (sr + r as u16, sg + g as u16, sb + b as u16)
            });
        let sample_area = (sample_size * sample_size) as u16;
        Color::Rgb(
            (sr / sample_area) as u8,
            (sg / sample_area) as u8,
            (sb / sample_area) as u8,
        )
    }
}

impl<'a, S: Scene, V: Viewport> tui::widgets::Widget for RasterWidget<'a, S, V> {
    fn render(self, area: tui::layout::Rect, buf: &mut tui::buffer::Buffer) {
        if area.area() == 0 {
            return;
        }

        let camera = V::new(
            self.scene.camera_transform().clone(),
            SCENE_WORLD_UNITS_PER_PIXEL * area.width as f64,
            SCENE_WORLD_UNITS_PER_PIXEL * area.height as f64 * 2.0,
        );

        self.raster.rasterize(
            self.scene,
            camera,
            self.aliasing * area.width as usize,
            self.aliasing * area.height as usize * 2,
        );

        for y in 0..area.height {
            for x in 0..area.width {
                let top_color = self.sample_color(x as usize, y as usize * 2);
                let bottom_color = self.sample_color(x as usize, y as usize * 2 + 1);

                let cell = buf.get_mut(x, y);
                cell.set_symbol("▄");
                cell.set_bg(top_color);
                cell.set_fg(bottom_color);
            }
        }
    }
}

#[derive(Default)]
pub struct Buffer2D<T> {
    pub width: usize,
    pub height: usize,
    pub data: Vec<T>,
}

impl<T> Buffer2D<T> {
    pub fn clear_and_resize(&mut self, width: usize, height: usize, default: T)
    where
        T: Copy,
    {
        let current_len = self.data.len();
        let desired_len = width * height;
        self.data.resize(desired_len, default);
        self.data[..current_len.min(desired_len)].fill(default);

        self.width = width;
        self.height = height;
    }

    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.data[y * self.width + x]
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        &mut self.data[y * self.width + x]
    }

    pub fn get_range(&self, y: usize, r: RangeInclusive<usize>) -> &[T] {
        &self.data[y * self.width + r.start()..=y * self.width + r.end()]
    }

    pub fn get_range_mut(&mut self, y: usize, r: RangeInclusive<usize>) -> &mut [T] {
        &mut self.data[y * self.width + r.start()..=y * self.width + r.end()]
    }

    pub fn area(
        &self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> impl Iterator<Item = impl Iterator<Item = &T>> {
        (y..y + height).map(move |y| self.get_range(y, x..=x + width - 1).iter())
    }
}
