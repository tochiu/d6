use crate::{transform::*, raster::SceneTriangle};

use tui::style::Color;

#[derive(Default)]
pub struct Body {
    pub transform: Transform,
    pub half_size: Vector3,
    pub linear_velocity: Vector3,
    pub angular_velocity: Vector3,
}

impl Body {
    pub fn geometry(&self) -> impl Iterator<Item = SceneTriangle> + '_ {
        BODY_UNIT_GEOMETRY.iter().map(|geometry| {
            let normal = self.transform.rotation.vector_to_world_space(geometry.normal);
            SceneTriangle {
                normal,
                points: geometry.points.map(|point| self.transform.point_to_world_space(point * self.half_size)),
                color: geometry.color
            }
        })
    }
}

// SAT collision test by checking for separating planes in all 15 axes
pub fn is_colliding(box1: &Body, box2: &Body) -> bool {
    let r_pos = box2.transform.position - box1.transform.position;

    let (box1_x_axis, box1_y_axis, box1_z_axis) = box1.transform.rotation.basis_vectors();
    let (box2_x_axis, box2_y_axis, box2_z_axis) = box2.transform.rotation.basis_vectors();

    !(get_separating_plane(r_pos, box1_x_axis, box1, box2)
        || get_separating_plane(r_pos, box1_y_axis, box1, box2)
        || get_separating_plane(r_pos, box1_z_axis, box1, box2)
        || get_separating_plane(r_pos, box2_x_axis, box1, box2)
        || get_separating_plane(r_pos, box2_y_axis, box1, box2)
        || get_separating_plane(r_pos, box2_z_axis, box1, box2)
        || get_separating_plane(r_pos, box1_x_axis.cross(box2_x_axis), box1, box2)
        || get_separating_plane(r_pos, box1_x_axis.cross(box2_y_axis), box1, box2)
        || get_separating_plane(r_pos, box1_x_axis.cross(box2_z_axis), box1, box2)
        || get_separating_plane(r_pos, box1_y_axis.cross(box2_x_axis), box1, box2)
        || get_separating_plane(r_pos, box1_y_axis.cross(box2_y_axis), box1, box2)
        || get_separating_plane(r_pos, box1_y_axis.cross(box2_z_axis), box1, box2)
        || get_separating_plane(r_pos, box1_z_axis.cross(box2_x_axis), box1, box2)
        || get_separating_plane(r_pos, box1_z_axis.cross(box2_y_axis), box1, box2)
        || get_separating_plane(r_pos, box1_z_axis.cross(box2_z_axis), box1, box2))
}

fn get_separating_plane(r_pos: Vector3, plane: Vector3, box1: &Body, box2: &Body) -> bool {
    let (box1_x_axis, box1_y_axis, box1_z_axis) = box1.transform.rotation.basis_vectors();
    let (box2_x_axis, box2_y_axis, box2_z_axis) = box2.transform.rotation.basis_vectors();

    return r_pos.dot(plane).abs()
        > (box1_x_axis * box1.half_size.x).dot(plane).abs()
            + (box1_y_axis * box1.half_size.y).dot(plane).abs()
            + (box1_z_axis * box1.half_size.z).dot(plane).abs()
            + (box2_x_axis * box2.half_size.x).dot(plane).abs()
            + (box2_y_axis * box2.half_size.y).dot(plane).abs()
            + (box2_z_axis * box2.half_size.z).dot(plane).abs();
}

pub const BODY_UNIT_GEOMETRY: [SceneTriangle; 12] = [
    // Front 0
    SceneTriangle {
        normal: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        points: [
            Vector3 {
                x: 1.000,
                y: 1.000,
                z: 1.0,
            },
            Vector3 {
                x: -1.000,
                y: 1.000,
                z: 1.0,
            },
            Vector3 {
                x: -1.000,
                y: -1.000,
                z: 1.0,
            },
        ],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Front 1
        normal: Vector3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        points: [
            Vector3 {
                x: 1.000,
                y: 1.000,
                z: 1.0,
            },
            Vector3 {
                x: -1.000,
                y: -1.000,
                z: 1.0,
            },
            Vector3 {
                x: 1.000,
                y: -1.000,
                z: 1.0,
            },
        ],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Back 0
        normal: Vector3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        },
        points: [
            Vector3 {
                x: 1.000,
                y: 1.000,
                z: -1.0,
            },
            Vector3 {
                x: -1.000,
                y: 1.000,
                z: -1.0,
            },
            Vector3 {
                x: -1.000,
                y: -1.000,
                z: -1.0,
            },
        ],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Back 1
        normal: Vector3 {
            x: 0.0,
            y: 0.0,
            z: -1.0,
        },
        points: [
        Vector3 {
            x: 1.000,
            y: 1.000,
            z: -1.0,
        },
        Vector3 {
            x: -1.000,
            y: -1.000,
            z: -1.0,
        },
        Vector3 {
            x: 1.000,
            y: -1.000,
            z: -1.0,
        }],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Right 0
        normal: Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        points: [Vector3 {
            x: 1.0,
            y: 1.000,
            z: 1.000,
        },
        Vector3 {
            x: 1.0,
            y: -1.000,
            z: 1.000,
        },
        Vector3 {
            x: 1.0,
            y: -1.000,
            z: -1.000,
        }],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Right 1
        normal: Vector3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        },
        points: [Vector3 {
            x: 1.0,
            y: 1.000,
            z: 1.000,
        },
        Vector3 {
            x: 1.0,
            y: -1.000,
            z: -1.000,
        },
        Vector3 {
            x: 1.0,
            y: 1.000,
            z: -1.000,
        },],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Left 0
        normal: Vector3 {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        },
        points: [Vector3 {
            x: -1.0,
            y: 1.000,
            z: 1.000,
        },
        Vector3 {
            x: -1.0,
            y: -1.000,
            z: 1.000,
        },
        Vector3 {
            x: -1.0,
            y: -1.000,
            z: -1.000,
        },],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Left 1
        normal: Vector3 {
            x: -1.0,
            y: 0.0,
            z: 0.0,
        },
        points: [Vector3 {
            x: -1.0,
            y: 1.000,
            z: 1.000,
        },
        Vector3 {
            x: -1.0,
            y: -1.000,
            z: -1.000,
        },
        Vector3 {
            x: -1.0,
            y: 1.000,
            z: -1.000,
        },],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Up 0
        normal: Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        points: [Vector3 {
            x: 1.000,
            y: 1.0,
            z: 1.000,
        },
        Vector3 {
            x: -1.000,
            y: 1.0,
            z: 1.000,
        },
        Vector3 {
            x: -1.000,
            y: 1.0,
            z: -1.000,
        },],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Up 1
        normal: Vector3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        },
        points: [Vector3 {
            x: 1.000,
            y: 1.0,
            z: 1.000,
        },
        Vector3 {
            x: -1.000,
            y: 1.0,
            z: -1.000,
        },
        Vector3 {
            x: 1.000,
            y: 1.0,
            z: -1.000,
        },],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Down 0
        normal: Vector3 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        },
        points: [Vector3 {
            x: 1.000,
            y: -1.0,
            z: 1.000,
        },
        Vector3 {
            x: -1.000,
            y: -1.0,
            z: 1.000,
        },
        Vector3 {
            x: -1.000,
            y: -1.0,
            z: -1.000,
        },],
        color: Color::Rgb(255, 255, 255),
    },
    SceneTriangle {
        // Down 1
        normal: Vector3 {
            x: 0.0,
            y: -1.0,
            z: 0.0,
        },
        points: [Vector3 {
            x: 1.000,
            y: -1.0,
            z: 1.000,
        },
        Vector3 {
            x: -1.000,
            y: -1.0,
            z: -1.000,
        },
        Vector3 {
            x: 1.000,
            y: -1.0,
            z: -1.000,
        },],
        color: Color::Rgb(255, 255, 255),
    },
];