//! `VisualsPlugin`: draws bodies/links/boundary/free-particles/VFX via `Gizmos`, plus a
//! full telemetry HUD (specs/transform/sandbox-hud.spec.md) and an interactive
//! click-to-burst (vfx-particle-system.spec.md §5). Requires rendering — only added in
//! `main.rs`, never in headless tests.

use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::body::{Anchor, Body};
use crate::config::{ARENA_RADIUS, BODY_VISUAL_RADIUS};
use crate::constraint::ConstraintLink;
use crate::cursor::cursor_world_position;
use crate::frame_budget::FrameBudget;
use crate::free_particles::FreeParticle;
use crate::rng::DeterministicRng;
use crate::vfx::{try_spawn_burst, VfxParticle};

/// sandbox-hud.spec.md §4: three-tier thresholds for the frame-average metric.
const FRAME_WARN_MS: f64 = 16.67;
const FRAME_CRIT_MS: f64 = 33.33;
/// sandbox-hud.spec.md §3: histogram cap and reference lines.
const HISTOGRAM_CAP_MS: f64 = 50.0;
const HISTOGRAM_TARGET_MS: f64 = 16.667;
const HISTOGRAM_THIRTY_HZ_MS: f64 = 33.333;
const HISTOGRAM_HEIGHT_PX: f32 = 88.0;
const HISTOGRAM_BAR_MAX_PX: f32 = 60.0;
/// Interactive burst sizes (vfx-particle-system.spec.md §5).
const CLICK_PARTICLE_BURST: usize = 12;
const CLICK_VFX_BURST: usize = 24;
const CLICK_VFX_LIFETIME_SECONDS: f64 = 0.6;
const CLICK_VFX_COLOR: [f32; 4] = [0.6, 0.8, 1.0, 1.0];

fn budget_color(avg_ms: f64) -> Color {
    if avg_ms > FRAME_CRIT_MS {
        Color::srgb(1.0, 0.47, 0.35)
    } else if avg_ms > FRAME_WARN_MS {
        Color::srgb(1.0, 0.86, 0.35)
    } else {
        Color::srgb(0.78, 0.82, 0.86)
    }
}

/// Deterministic fixed-tick counter for the HUD's telemetry line — mirrors
/// `game::GameTick` but scoped to this scene.
#[derive(Resource, Debug, Default)]
struct SimTick(u64);

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimTick::default())
            .add_systems(Startup, (spawn_camera, spawn_hud))
            .add_systems(FixedUpdate, tick_counter)
            .add_systems(
                Update,
                (
                    draw_boundary,
                    draw_bodies,
                    draw_links,
                    draw_free_particles,
                    draw_vfx_particles,
                    click_to_burst,
                    update_telemetry_text,
                    update_counts_text,
                    update_histogram,
                ),
            );
    }
}

fn tick_counter(mut tick: ResMut<SimTick>) {
    tick.0 += 1;
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

fn draw_free_particles(mut gizmos: Gizmos, particles: Query<&FreeParticle>) {
    for particle in &particles {
        gizmos.circle_2d(
            particle.position.as_vec2(),
            particle.radius as f32,
            css::CORNFLOWER_BLUE,
        );
    }
}

fn draw_vfx_particles(mut gizmos: Gizmos, particles: Query<&VfxParticle>) {
    for particle in &particles {
        let color = Color::srgba(
            particle.color[0],
            particle.color[1],
            particle.color[2],
            particle.color[3],
        );
        gizmos.circle_2d(particle.position.as_vec3().truncate(), particle.size, color);
    }
}

/// Interactive burst (vfx-particle-system.spec.md §5): right-click spawns a physics
/// particle burst plus a VFX burst at the cursor, additive to the ambient bounce sparks.
fn click_to_burst(
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut commands: Commands,
    vfx_particles: Query<(), With<VfxParticle>>,
    mut rng: ResMut<DeterministicRng>,
) {
    if !mouse.just_pressed(MouseButton::Right) {
        return;
    }
    let Some(cursor_world) = cursor_world_position(&windows, &cameras) else {
        return;
    };
    let cursor_world = bevy::math::DVec2::new(f64::from(cursor_world.x), f64::from(cursor_world.y));

    // A small physics burst: spawn is done by pushing new entities directly here since
    // FreeParticle has no dedicated spawn helper outside Startup — mirrors the
    // reference's spawn_particle_burst (random outward velocity/radius).
    use rand::Rng;
    for _ in 0..CLICK_PARTICLE_BURST {
        let angle = rng.0.random_range(0.0..std::f64::consts::TAU);
        let speed = rng.0.random_range(20.0..120.0);
        commands.spawn(FreeParticle {
            position: cursor_world,
            velocity: bevy::math::DVec2::new(angle.cos() * speed, angle.sin() * speed),
            radius: rng.0.random_range(2.5..5.5),
        });
    }

    let live_vfx = vfx_particles.iter().count();
    try_spawn_burst(
        &mut commands,
        live_vfx,
        cursor_world.extend(0.0),
        CLICK_VFX_BURST,
        CLICK_VFX_LIFETIME_SECONDS,
        CLICK_VFX_COLOR,
        &mut rng,
    );
}

#[derive(Component)]
struct TelemetryText;
#[derive(Component)]
struct CountsText;
#[derive(Component)]
struct HistogramBar;
#[derive(Component)]
struct HistogramAvgText;

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
                Text::new("ORBITAL_ARENA_RS showcase - constraint"),
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
        });

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
                Text::new("RMB burst (physics + VFX)   Esc quit"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.67, 0.71, 0.78)),
            ));
        });

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
    config: Res<crate::config::SimConfig>,
    tick: Res<SimTick>,
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
        "seed={}  substeps={}  frame_avg={avg:.2}ms  fps={fps}",
        config.seed, tick.0
    );
    *color = TextColor(budget_color(avg));
}

fn update_counts_text(
    bodies: Query<&Body>,
    links: Query<&ConstraintLink>,
    free_particles: Query<&FreeParticle>,
    vfx_particles: Query<&VfxParticle>,
    mut text: Query<&mut Text, With<CountsText>>,
) {
    let Ok(mut text) = text.single_mut() else {
        return;
    };
    **text = format!(
        "bodies={}  edges={}  particles={}  vfx={}",
        bodies.iter().count(),
        links.iter().count(),
        free_particles.iter().count(),
        vfx_particles.iter().count()
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
