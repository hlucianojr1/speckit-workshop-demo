//! Visuals for the Orbital Arena game scene: wells, particle field, arena bounds, and a
//! full telemetry HUD matching specs/transform/sandbox-hud.spec.md (reverse-specced from
//! `apps/sandbox/app.cpp`'s `draw_hud`/`draw_histogram`: title, seed/tick line, frame-time
//! line with three-tier color coding, per-player scores, winner banner, top-right counts
//! panel, control-hint legend, and a frame-budget histogram widget). Requires rendering —
//! only added in `main.rs`, never in headless tests.

use bevy::color::palettes::css;
use bevy::prelude::*;

use crate::config::SimConfig;
use crate::frame_budget::FrameBudget;

use super::scoring::ScoreTable;
use super::well::GravityWell;
use super::{FieldParticle, GameTick, HALF_EXTENT, PARTICLE_COUNT, WELL_COUNT};

const WELL_COLORS: [Srgba; 4] = [css::DODGER_BLUE, css::ORANGE_RED, css::LIME, css::MAGENTA];

/// sandbox-hud.spec.md §4: three-tier thresholds for the frame-average metric.
const FRAME_WARN_MS: f64 = 16.67;
const FRAME_CRIT_MS: f64 = 33.33;
/// sandbox-hud.spec.md §3: histogram cap and reference lines.
const HISTOGRAM_CAP_MS: f64 = 50.0;
const HISTOGRAM_TARGET_MS: f64 = 16.667;
const HISTOGRAM_THIRTY_HZ_MS: f64 = 33.333;
const HISTOGRAM_HEIGHT_PX: f32 = 88.0;
const HISTOGRAM_BAR_MAX_PX: f32 = 60.0;

/// sandbox-hud.spec.md §4's three-tier color pattern, shared by the frame-average text
/// and the histogram bar.
fn budget_color(avg_ms: f64) -> Color {
    if avg_ms > FRAME_CRIT_MS {
        Color::srgb(1.0, 0.47, 0.35)
    } else if avg_ms > FRAME_WARN_MS {
        Color::srgb(1.0, 0.86, 0.35)
    } else {
        Color::srgb(0.78, 0.82, 0.86)
    }
}

pub struct GameVisualsPlugin;

impl Plugin for GameVisualsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (spawn_camera, spawn_hud))
            .add_systems(
                Update,
                (
                    draw_boundary,
                    draw_particles,
                    draw_wells,
                    update_telemetry_text,
                    update_score_text,
                    update_winner_text,
                    update_counts_text,
                    update_histogram,
                ),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

#[derive(Component)]
struct TelemetryText;
#[derive(Component)]
struct ScoreText(u8);
#[derive(Component)]
struct WinnerText;
#[derive(Component)]
struct CountsText;
#[derive(Component)]
struct HistogramBar;
#[derive(Component)]
struct HistogramAvgText;

/// sandbox-hud.spec.md §2: top-left title/telemetry column, top-right counts panel,
/// bottom-left control hints, bottom-right frame-budget histogram.
fn spawn_hud(mut commands: Commands) {
    // Top-left: title + telemetry + per-player scores + winner banner.
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
                Text::new("ORBITAL_ARENA_RS showcase - arena"),
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
            for player in 0..WELL_COUNT as u8 {
                let color = WELL_COLORS
                    .get(player as usize % WELL_COLORS.len())
                    .copied()
                    .unwrap_or(css::WHITE);
                parent.spawn((
                    Text::new(format!("P{player} score: 0")),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(color.into()),
                    ScoreText(player),
                ));
            }
            parent.spawn((
                Text::new(String::new()),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                WinnerText,
            ));
        });

    // Top-right: boxed counts panel.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(12.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(String::new()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.9, 0.94)),
                CountsText,
            ));
        });

    // Bottom-left: control hints (sandbox-hud.spec.md §2 bottom-left lines).
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(24.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(2.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text::new("--scene arena|constraint   --seed <u64>   screenshot subcommand"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.67, 0.71, 0.78)),
            ));
            parent.spawn((
                Text::new("LMB drag P0's well (near it)   Esc quit"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.67, 0.71, 0.78)),
            ));
        });

    // Bottom-right: frame-budget histogram widget (sandbox-hud.spec.md §3).
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(20.0),
                right: Val::Px(20.0),
                width: Val::Px(160.0),
                height: Val::Px(HISTOGRAM_HEIGHT_PX),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("frame budget (ms)"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.71, 0.78, 0.94)),
            ));

            // Reference lines (static offsets — target/cap are compile-time constants).
            let inner_height = HISTOGRAM_BAR_MAX_PX;
            let sixty_hz_bottom = ((HISTOGRAM_TARGET_MS / HISTOGRAM_CAP_MS) as f32) * inner_height;
            let thirty_hz_bottom =
                ((HISTOGRAM_THIRTY_HZ_MS / HISTOGRAM_CAP_MS) as f32) * inner_height;
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0 + sixty_hz_bottom),
                    left: Val::Px(8.0),
                    width: Val::Px(140.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.47, 0.86, 0.47, 0.78)),
            ));
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0 + thirty_hz_bottom),
                    left: Val::Px(8.0),
                    width: Val::Px(140.0),
                    height: Val::Px(1.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.86, 0.35, 0.35, 0.78)),
            ));

            // The bar itself: pinned to the bottom, height set every frame.
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    left: Val::Px(12.0),
                    width: Val::Px(24.0),
                    height: Val::Px(0.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.47, 0.86, 0.47)),
                HistogramBar,
            ));

            parent.spawn((
                Text::new(String::new()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.86, 0.9, 0.94)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(2.0),
                    left: Val::Px(8.0),
                    ..default()
                },
                HistogramAvgText,
            ));
        });
}

fn update_telemetry_text(
    config: Res<SimConfig>,
    tick: Res<GameTick>,
    budget: Res<FrameBudget>,
    mut text: Query<(&mut Text, &mut TextColor), With<TelemetryText>>,
) {
    let Ok((mut text, mut color)) = text.single_mut() else {
        return;
    };
    let avg = budget.rolling_average();
    let fps = if avg > 0.0 {
        (1000.0 / avg).round() as i64
    } else {
        0
    };
    **text = format!(
        "seed={}  tick={}  frame_avg={avg:.2}ms  fps={fps}",
        config.seed, tick.0
    );
    *color = TextColor(budget_color(avg));
}

fn update_score_text(scores: Res<ScoreTable>, mut texts: Query<(&ScoreText, &mut Text)>) {
    for (marker, mut text) in &mut texts {
        if let Some(score) = scores.scores.get(marker.0 as usize) {
            **text = format!("P{} score: {score}", marker.0);
        }
    }
}

fn update_winner_text(scores: Res<ScoreTable>, mut text: Query<&mut Text, With<WinnerText>>) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    **text = match scores.winner {
        Some(winner) => format!("WINNER: P{winner}"),
        None => String::new(),
    };
}

fn update_counts_text(
    wells: Query<&GravityWell>,
    particles: Query<&FieldParticle>,
    mut text: Query<&mut Text, With<CountsText>>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    **text = format!(
        "wells={}  particles={}  (cap={})",
        wells.iter().count(),
        particles.iter().count(),
        PARTICLE_COUNT
    );
}

fn update_histogram(
    budget: Res<FrameBudget>,
    mut bar: Query<(&mut Node, &mut BackgroundColor), With<HistogramBar>>,
    mut avg_text: Query<&mut Text, With<HistogramAvgText>>,
) {
    let avg = budget.rolling_average();
    let bar_height = ((avg / HISTOGRAM_CAP_MS).clamp(0.0, 1.0) as f32) * HISTOGRAM_BAR_MAX_PX;
    if let Ok((mut node, mut color)) = bar.single_mut() {
        node.height = Val::Px(bar_height);
        *color = BackgroundColor(budget_color(avg));
    }
    if let Ok(mut text) = avg_text.single_mut() {
        **text = format!("{avg:.2} ms  avg");
    }
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
