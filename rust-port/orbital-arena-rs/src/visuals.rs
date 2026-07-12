//! `VisualsPlugin`: draws bodies/links/free-particles/VFX via `Gizmos`, plus a full
//! telemetry HUD (specs/transform/sandbox-hud.spec.md) and an interactive click-to-burst
//! (vfx-particle-system.spec.md §5). Requires rendering — only added in `main.rs`, never
//! in headless tests. No arena-boundary circle here — that's the `arena` scene's own
//! rectangular bounds (`game::visuals::draw_boundary`); this scene's reference has no
//! boundary shape at all (sandbox-visual-identity.spec.md §1–§9 lists none for it).

use bevy::math::DVec2;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::body::{Anchor, Body};
use crate::color_ramp::color_from_speed;
use crate::constraint::ConstraintLink;
use crate::cursor::cursor_world_position;
use crate::frame_budget::FrameBudget;
use crate::free_particles::{FreeParticle, Trail};
use crate::rng::DeterministicRng;
use crate::vfx::{try_spawn_burst, VfxParticle, VFX_LIFETIME_SECONDS, VFX_SPAWN_COLOR};

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

/// A second `GizmoConfigGroup` so edge/link glow passes can use a wider line than the
/// default group's core pass (sandbox-visual-identity.spec.md §5: glow 4 px, core 2 px).
#[derive(Default, Reflect, GizmoConfigGroup)]
#[reflect(Default)]
pub struct GlowGizmoConfigGroup;

fn budget_color(avg_ms: f64) -> Color {
    if avg_ms > FRAME_CRIT_MS {
        Color::srgb(1.0, 120.0 / 255.0, 90.0 / 255.0)
    } else if avg_ms > FRAME_WARN_MS {
        Color::srgb(1.0, 220.0 / 255.0, 90.0 / 255.0)
    } else {
        Color::srgb(200.0 / 255.0, 210.0 / 255.0, 220.0 / 255.0)
    }
}

/// sandbox-visual-identity.spec.md §9's HISTOGRAM BAR palette — distinct from
/// `budget_color`'s frame-average TEXT palette above (green/amber/red, not
/// neutral/amber/red, and always α 230).
fn histogram_bar_color(avg_ms: f64) -> Color {
    if avg_ms > FRAME_CRIT_MS {
        Color::srgba(220.0 / 255.0, 90.0 / 255.0, 90.0 / 255.0, 230.0 / 255.0)
    } else if avg_ms > FRAME_WARN_MS {
        Color::srgba(220.0 / 255.0, 200.0 / 255.0, 90.0 / 255.0, 230.0 / 255.0)
    } else {
        Color::srgba(120.0 / 255.0, 220.0 / 255.0, 120.0 / 255.0, 230.0 / 255.0)
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
            .init_gizmo_group::<GlowGizmoConfigGroup>()
            .add_systems(Startup, (spawn_hud, configure_gizmo_line_widths))
            .add_systems(FixedUpdate, tick_counter)
            .add_systems(
                Update,
                (
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

/// sandbox-visual-identity.spec.md §5: edge glow 4 px, edge core 2 px.
fn configure_gizmo_line_widths(mut store: ResMut<GizmoConfigStore>) {
    store.config_mut::<DefaultGizmoConfigGroup>().0.line.width = 2.0;
    store.config_mut::<GlowGizmoConfigGroup>().0.line.width = 4.0;
}

/// §5: anchor is a 9-px halo + centered 10×10 golden square; dynamic nodes are three
/// concentric glow circles. Gizmos has no filled-rect primitive, so the "square" is an
/// outline rect at the reference's exact size — see the module doc for the filled-shape
/// limitation this port accepts throughout (Bevy Gizmos draws outlines, not fills).
fn draw_bodies(mut gizmos: Gizmos, bodies: Query<(&Body, Has<Anchor>)>) {
    for (body, is_anchor) in &bodies {
        let position = body.position.as_vec2();
        if is_anchor {
            gizmos.circle_2d(position, 0.09, Color::srgba(1.0, 0.78, 0.24, 40.0 / 255.0));
            gizmos.rect_2d(position, Vec2::splat(0.10), Color::srgb(1.0, 0.82, 0.47));
        } else {
            gizmos.circle_2d(position, 0.10, Color::srgba(0.39, 0.78, 1.0, 30.0 / 255.0));
            gizmos.circle_2d(position, 0.06, Color::srgba(0.55, 0.86, 1.0, 80.0 / 255.0));
            gizmos.circle_2d(position, 0.04, Color::srgba(0.78, 0.96, 1.0, 230.0 / 255.0));
        }
    }
}

/// §5: edge glow (wide, faint) pass, then a narrower, brighter core pass — two distinct
/// `GizmoConfigGroup`s so each pass keeps its own line width (see `GlowGizmoConfigGroup`
/// in `main.rs`/plugin registration).
fn draw_links(
    mut glow: Gizmos<GlowGizmoConfigGroup>,
    mut core: Gizmos,
    links: Query<&ConstraintLink>,
    bodies: Query<&Body>,
) {
    for link in &links {
        let (Ok(a), Ok(b)) = (bodies.get(link.a), bodies.get(link.b)) else {
            continue;
        };
        let (a, b) = (a.position.as_vec2(), b.position.as_vec2());
        glow.line_2d(
            a,
            b,
            Color::srgba(100.0 / 255.0, 160.0 / 255.0, 1.0, 40.0 / 255.0),
        );
        core.line_2d(
            a,
            b,
            Color::srgba(200.0 / 255.0, 220.0 / 255.0, 1.0, 230.0 / 255.0),
        );
    }
}

/// §4: 4-layer bloom (outer/mid/core/hot-center) colored by the speed ramp, plus a
/// fading trail. Gizmos lines have one global width per config group (no per-call
/// variable width), so the reference's segment WIDTH taper (2.5→0.5 px) is not
/// reproduced literally here — only its ALPHA taper (`progress·180`) and per-segment
/// speed coloring are, which is what makes the trail visually read as "fading".
fn draw_free_particles(
    mut gizmos: Gizmos,
    time: Res<Time>,
    particles: Query<(&FreeParticle, &Trail)>,
) {
    let t = time.elapsed_secs_f64();
    for (i, (particle, trail)) in particles.iter().enumerate() {
        let speed = particle.velocity.length();
        let pulse = 1.0 + 0.3 * (4.0 * t + 0.7 * i as f64).sin();
        let r = particle.radius * pulse;

        let points: Vec<DVec2> = trail.iter_oldest_to_newest().collect();
        for (k, window) in points.windows(2).enumerate() {
            let [from, to] = window else { continue };
            let progress = (k + 1) as f64 / Trail::LEN as f64;
            let segment_speed = (*to - *from).length() * 60.0;
            let alpha = (progress * 180.0 / 255.0) as f32;
            let color = color_from_speed(segment_speed, alpha);
            gizmos.line_2d(from.as_vec2(), to.as_vec2(), color);
        }

        let core_color = color_from_speed(speed, 1.0);
        gizmos.circle_2d(
            particle.position.as_vec2(),
            (4.0 * r) as f32,
            core_color.with_alpha(20.0 / 255.0),
        );
        gizmos.circle_2d(
            particle.position.as_vec2(),
            (2.2 * r) as f32,
            core_color.with_alpha(50.0 / 255.0),
        );
        gizmos.circle_2d(particle.position.as_vec2(), r as f32, core_color);
        if speed > 3.0 {
            let hot_alpha = ((speed - 3.0) / 5.0).clamp(0.0, 1.0) as f32;
            gizmos.circle_2d(
                particle.position.as_vec2(),
                (0.4 * r) as f32,
                Color::srgba(1.0, 1.0, 1.0, hot_alpha),
            );
        }
    }
}

/// §6: VFX particles render with a fixed two-layer warm-orange palette (glow + core) —
/// the reference's one shared emitter always spawns this color (§9), so the render pass
/// uses these literal values rather than each particle's stored `color` field. The
/// reference computes `r = max(size·60, 1.5)` PIXELS for its direct-pixel-space
/// renderer; since this port draws radii directly in WORLD units (the camera's own
/// scale handles the px conversion — see `background.rs`), the equivalent minimum is
/// expressed in world units instead of re-deriving a pixel scale here.
const MIN_VFX_RADIUS_WORLD: f32 = 0.008;

fn draw_vfx_particles(mut gizmos: Gizmos, particles: Query<&VfxParticle>) {
    for particle in &particles {
        let life_fraction =
            (particle.remaining_lifetime_seconds / VFX_LIFETIME_SECONDS).clamp(0.0, 1.0);
        let alpha = (life_fraction * 220.0 / 255.0) as f32;
        let r = particle.size.max(MIN_VFX_RADIUS_WORLD);
        let position = particle.position.as_vec3().truncate();
        gizmos.circle_2d(
            position,
            2.0 * r,
            Color::srgba(1.0, 210.0 / 255.0, 110.0 / 255.0, alpha / 3.0),
        );
        gizmos.circle_2d(
            position,
            r,
            Color::srgba(1.0, 225.0 / 255.0, 140.0 / 255.0, alpha),
        );
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
    // reference's spawn_particle_burst(wx, wy, n) (sandbox-scenes.spec.md §7): per
    // particle, angle = u·2π, speed = 1.5 + u·2.5, radius = 0.04 + u·0.03 (world units —
    // NOT the pixel-scale placeholder values this used before the rope/world-scale
    // rewrite (T048–T052) left them stale).
    use rand::Rng;
    for _ in 0..CLICK_PARTICLE_BURST {
        let angle = rng.0.random_range(0.0..std::f64::consts::TAU);
        let speed = 1.5 + rng.0.random_range(0.0..1.0) * 2.5;
        commands.spawn((
            FreeParticle {
                position: cursor_world,
                velocity: bevy::math::DVec2::new(angle.cos() * speed, angle.sin() * speed),
                radius: 0.04 + rng.0.random_range(0.0..1.0) * 0.03,
            },
            Trail::new(cursor_world),
        ));
    }

    let live_vfx = vfx_particles.iter().count();
    try_spawn_burst(
        &mut commands,
        live_vfx,
        cursor_world.extend(0.0),
        CLICK_VFX_BURST,
        VFX_LIFETIME_SECONDS,
        VFX_SPAWN_COLOR,
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
                TextColor(Color::srgb(1.0, 240.0 / 255.0, 200.0 / 255.0)),
            ));
            parent.spawn((
                Text::new(String::new()),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(245.0 / 255.0, 245.0 / 255.0, 245.0 / 255.0)),
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
                TextColor(Color::srgb(220.0 / 255.0, 230.0 / 255.0, 240.0 / 255.0)),
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
                TextColor(Color::srgba(
                    170.0 / 255.0,
                    180.0 / 255.0,
                    200.0 / 255.0,
                    220.0 / 255.0,
                )),
            ));
            parent.spawn((
                Text::new("RMB burst (physics + VFX)   Esc quit"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(
                    170.0 / 255.0,
                    180.0 / 255.0,
                    200.0 / 255.0,
                    220.0 / 255.0,
                )),
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
                TextColor(Color::srgb(180.0 / 255.0, 200.0 / 255.0, 240.0 / 255.0)),
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
                TextColor(Color::srgb(220.0 / 255.0, 220.0 / 255.0, 240.0 / 255.0)),
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
        *color = BackgroundColor(histogram_bar_color(avg));
    }
    if let Ok(mut text) = avg_text.single_mut() {
        **text = format!("{avg:.2} ms  avg");
    }
}
