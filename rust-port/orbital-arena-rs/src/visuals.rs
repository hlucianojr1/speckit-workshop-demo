//! `VisualsPlugin`: draws bodies/links/boundary via `Gizmos` (contracts/plugins.md
//! `VisualsPlugin`, research.md R7). Requires rendering — only added in `main.rs`, never
//! in headless tests.

use bevy::color::palettes::css;
use bevy::prelude::*;

use crate::body::{Anchor, Body};
use crate::config::{ARENA_RADIUS, BODY_VISUAL_RADIUS};
use crate::constraint::ConstraintLink;

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (draw_boundary, draw_bodies, draw_links));
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn draw_boundary(mut gizmos: Gizmos) {
    gizmos.circle_2d(Vec2::ZERO, ARENA_RADIUS as f32, css::DIM_GRAY);
}

fn draw_bodies(mut gizmos: Gizmos, bodies: Query<(&Body, Has<Anchor>)>) {
    for (body, is_anchor) in &bodies {
        let position = body.position.as_vec2();
        if is_anchor {
            gizmos.circle_2d(position, BODY_VISUAL_RADIUS as f32 * 1.5, css::GOLD);
        } else {
            gizmos.circle_2d(position, BODY_VISUAL_RADIUS as f32, css::DODGER_BLUE);
        }
    }
}

fn draw_links(mut gizmos: Gizmos, links: Query<&ConstraintLink>, bodies: Query<&Body>) {
    for link in &links {
        let (Ok(a), Ok(b)) = (bodies.get(link.a), bodies.get(link.b)) else {
            continue;
        };
        gizmos.line_2d(a.position.as_vec2(), b.position.as_vec2(), css::LIGHT_GREEN);
    }
}
