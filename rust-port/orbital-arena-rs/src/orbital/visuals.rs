//! Visuals for the Orbital Arena scene: camera, wells, particle field, arena bounds,
//! and a telemetry + match HUD (sandbox-hud.spec.md §2/§9 match panel). Requires
//! rendering — only added in `main.rs`, never in headless tests.

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

use crate::config::SimConfig;
use crate::frame_budget::FrameBudget;

use super::match_state::MatchState;
use super::ArenaRes;

/// Exact player colors (sandbox-visual-identity.spec.md §7).
const PLAYER_COLORS: [Srgba; 4] = [
    Srgba::new(80.0 / 255.0, 190.0 / 255.0, 1.0, 1.0), // P0 cyan
    Srgba::new(1.0, 150.0 / 255.0, 60.0 / 255.0, 1.0), // P1 orange
    Srgba::new(120.0 / 255.0, 230.0 / 255.0, 120.0 / 255.0, 1.0), // P2 green
    Srgba::new(210.0 / 255.0, 120.0 / 255.0, 1.0, 1.0), // P3 purple
];

/// sandbox-hud.spec.md §4: three-tier thresholds for the frame-average metric.
const FRAME_WARN_MS: f64 = 16.67;
const FRAME_CRIT_MS: f64 = 33.33;

fn budget_color(avg_ms: f64) -> Color {
    if avg_ms > FRAME_CRIT_MS {
        Color::srgb(1.0, 120.0 / 255.0, 90.0 / 255.0)
    } else if avg_ms > FRAME_WARN_MS {
        Color::srgb(1.0, 220.0 / 255.0, 90.0 / 255.0)
    } else {
        Color::srgb(200.0 / 255.0, 210.0 / 255.0, 220.0 / 255.0)
    }
}

pub struct ArenaVisualsPlugin;

impl Plugin for ArenaVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera, spawn_hud))
            .add_systems(
                Update,
                (draw_bounds, draw_particles, draw_wells, update_hud),
            );
    }
}

/// Calibrated to the sandbox embedding's `half_extent` (2.0, sandbox-scenes.spec.md
/// §4.5) with headroom for the influence-radius rings.
fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 5.0,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

#[derive(Component)]
struct TelemetryText;
#[derive(Component)]
struct MatchText;

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(4.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("ORBITAL_ARENA_RS showcase - orbital"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.94, 0.78)),
            ));
            parent.spawn((
                Text::new(String::new()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TelemetryText,
            ));
            parent.spawn((
                Text::new(String::new()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                MatchText,
            ));
        });
}

#[allow(clippy::type_complexity)]
fn update_hud(
    config: Res<SimConfig>,
    budget: Res<FrameBudget>,
    arena: Option<Res<ArenaRes>>,
    mut telemetry: Query<(&mut Text, &mut TextColor), (With<TelemetryText>, Without<MatchText>)>,
    mut match_text: Query<&mut Text, (With<MatchText>, Without<TelemetryText>)>,
) {
    let avg = budget.rolling_average();
    if let Ok((mut text, mut color)) = telemetry.single_mut() {
        let fps = if avg > 0.0 {
            (1000.0 / avg).round() as i64
        } else {
            0
        };
        **text = format!("seed={}  frame_avg={avg:.2}ms  fps={fps}", config.seed);
        *color = TextColor(budget_color(avg));
    }
    let Some(arena) = arena else {
        return;
    };
    let Ok(mut text) = match_text.single_mut() else {
        return;
    };
    let state_name = match arena.0.state() {
        MatchState::Lobby => "lobby",
        MatchState::Countdown => "countdown",
        MatchState::Playing => "playing",
        MatchState::GameOver => "game_over",
    };
    let mut line = format!("tick={}  state={state_name}", arena.0.current_tick());
    for (i, _) in arena.0.wells().iter().enumerate() {
        let score = arena.0.score(i as u8);
        line.push_str(&format!("  P{i}={score}"));
    }
    if let Some(winner) = arena.0.winner() {
        line.push_str(&format!("  WINNER: P{winner}"));
    }
    **text = line;
}

/// Bounds rectangle outline at world ±half_extent (sandbox-visual-identity.spec.md §7).
fn draw_bounds(mut gizmos: Gizmos, arena: Option<Res<ArenaRes>>) {
    let Some(arena) = arena else {
        return;
    };
    let half = arena.0.config().half_extent;
    gizmos.rect_2d(
        Vec2::ZERO,
        Vec2::splat(half * 2.0),
        Color::srgba(90.0 / 255.0, 110.0 / 255.0, 160.0 / 255.0, 200.0 / 255.0),
    );
}

/// Each well: a capture-radius core circle plus a faint influence-radius ring, in the
/// owning player's color (sandbox-visual-identity.spec.md §7).
fn draw_wells(mut gizmos: Gizmos, arena: Option<Res<ArenaRes>>) {
    let Some(arena) = arena else {
        return;
    };
    for (i, well) in arena.0.wells().iter().enumerate() {
        if !well.active {
            continue;
        }
        let color = PLAYER_COLORS
            .get(i % PLAYER_COLORS.len())
            .copied()
            .unwrap_or(Srgba::WHITE);
        gizmos.circle_2d(well.position, well.capture_radius, color);
        gizmos.circle_2d(well.position, well.influence_radius, color.with_alpha(0.15));
    }
}

/// Each field particle: neutral gray-blue when unclaimed, otherwise the color of the
/// nearest active well whose influence radius contains it (sandbox-visual-identity.spec.md §7).
fn draw_particles(mut gizmos: Gizmos, arena: Option<Res<ArenaRes>>) {
    const NEUTRAL: Srgba = Srgba::new(170.0 / 255.0, 180.0 / 255.0, 210.0 / 255.0, 200.0 / 255.0);
    let Some(arena) = arena else {
        return;
    };
    let wells = arena.0.wells();
    for particle in arena.0.live_particles() {
        let mut color = NEUTRAL;
        let mut nearest_sq = f32::MAX;
        for (i, well) in wells.iter().enumerate() {
            if !well.active {
                continue;
            }
            let distance_sq = (well.position - particle.position).length_squared();
            if distance_sq <= well.influence_radius * well.influence_radius
                && distance_sq < nearest_sq
            {
                nearest_sq = distance_sq;
                color = PLAYER_COLORS
                    .get(i % PLAYER_COLORS.len())
                    .copied()
                    .unwrap_or(NEUTRAL);
            }
        }
        gizmos.circle_2d(particle.position, 0.03, color);
    }
}
