//! Visuals for the Orbital Arena game scene: wells, particle field, arena bounds, and a
//! score HUD (contracts loosely mirroring `crate::visuals`, but a distinct plugin since
//! this renders a different scene). Requires rendering — only added in `main.rs`, never
//! in headless tests.

use bevy::color::palettes::css;
use bevy::prelude::*;

use super::scoring::ScoreTable;
use super::well::GravityWell;
use super::{FieldParticle, HALF_EXTENT};

const WELL_COLORS: [Srgba; 4] = [css::DODGER_BLUE, css::ORANGE_RED, css::LIME, css::MAGENTA];

pub struct GameVisualsPlugin;

impl Plugin for GameVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera, spawn_hud))
            .add_systems(
                Update,
                (draw_boundary, draw_particles, draw_wells, update_hud),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Component)]
struct HudText;

fn spawn_hud(mut commands: Commands) {
    commands.spawn((
        Text::new("P0: 0   P1: 0"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::WHITE),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
        HudText,
    ));
}

fn update_hud(scores: Res<ScoreTable>, mut text: Query<&mut Text, With<HudText>>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    let mut label = String::new();
    for (i, score) in scores.scores.iter().take(2).enumerate() {
        if i > 0 {
            label.push_str("   ");
        }
        label.push_str(&format!("P{i}: {score}"));
    }
    if let Some(winner) = scores.winner {
        label.push_str(&format!("   WINNER: P{winner}"));
    }
    **text = label;
}

fn draw_boundary(mut gizmos: Gizmos) {
    let half = HALF_EXTENT as f32;
    gizmos.rect_2d(Vec2::ZERO, Vec2::splat(half * 2.0), css::DIM_GRAY);
}

fn draw_particles(mut gizmos: Gizmos, particles: Query<&FieldParticle>) {
    for particle in &particles {
        gizmos.circle_2d(particle.position.as_vec2(), 2.5, css::LIGHT_GRAY);
    }
}

fn draw_wells(mut gizmos: Gizmos, wells: Query<&GravityWell>) {
    for well in &wells {
        let color = WELL_COLORS
            .get(well.player_index as usize % WELL_COLORS.len())
            .copied()
            .unwrap_or(css::WHITE);
        gizmos.circle_2d(well.position.as_vec2(), well.capture_radius as f32, color);
        gizmos.circle_2d(well.position.as_vec2(), 4.0, color);
    }
}
