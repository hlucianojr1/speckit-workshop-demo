//! Background visual identity for the rope/`constraint` scene
//! (sandbox-visual-identity.spec.md §1–§2): the world→screen camera mapping, the
//! deep-space gradient, the breathing world grid, the vignette, the world floor line,
//! and the 60 cosmetic dust motes. Calibrated for that scene's small `[-3.5, 3.5]`
//! world-unit extent — NOT added to the `arena` scene, which uses its own pixel-like
//! coordinate system (`HALF_EXTENT` ≈ 220, see `game::visuals::spawn_camera`).

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

/// World units spanning the window's full vertical extent. The rope's anchor sits at
/// `ROPE_ANCHOR_Y` (3.0) and the free-particle/floor area extends down to about −2.5, so
/// the reference frames considerably more than the spec §1 "~[−2.5, 2.5]" headline figure
/// (that range describes the general default framing, not this specific scene's tall
/// anchor-to-floor span) — a fixed VERTICAL extent keeps the whole rope on-screen
/// regardless of window aspect ratio, matching `docs/screenshots/spec-test-cpp-rope.png`.
const WORLD_VIEW_HEIGHT: f32 = 6.5;

/// Deep-space gradient bands (§2.1): top and bottom stops, in `[0, 1]` linear channels.
const GRADIENT_TOP: Vec3 = Vec3::new(8.0 / 255.0, 12.0 / 255.0, 28.0 / 255.0);
const GRADIENT_BOTTOM: Vec3 = Vec3::new(2.0 / 255.0, 2.0 / 255.0, 6.0 / 255.0);
const GRADIENT_BAND_COUNT: i32 = 32;
/// Half-height/width of the gradient backdrop in world units — generous enough to cover
/// any reasonable window aspect ratio at `WORLD_VIEW_WIDTH`.
const GRADIENT_HALF_EXTENT: f32 = 12.0;
/// Drawn behind every other world-space sprite/gizmo.
const BACKGROUND_Z: f32 = -100.0;

/// Breathing grid (§2.2): lines at these world coordinates.
const GRID_VERTICAL_LINES: [f32; 7] = [-3.0, -2.0, -1.0, 0.0, 1.0, 2.0, 3.0];
const GRID_HORIZONTAL_LINES: [f32; 5] = [-2.0, -1.0, 0.0, 1.0, 2.0];
const GRID_HALF_HEIGHT: f32 = 2.5;
const GRID_HALF_WIDTH: f32 = 3.5;

/// World floor reference line (§2.5).
const FLOOR_Y: f32 = -2.0;
const FLOOR_HALF_WIDTH: f32 = 3.0;

/// Dust motes (§2.4): fixed population, integer-hash seeded.
const DUST_MOTE_COUNT: u32 = 60;
const DUST_HASH_SEED_MULTIPLIER: u32 = 2_654_435_761;
const DUST_LCG_MULTIPLIER: u32 = 1_664_525;
const DUST_LCG_INCREMENT: u32 = 1_013_904_223;

/// Clear color (§1: (8, 10, 16)).
pub const CLEAR_COLOR: Color = Color::srgb(8.0 / 255.0, 10.0 / 255.0, 16.0 / 255.0);

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(CLEAR_COLOR))
            .add_systems(
                Startup,
                (
                    spawn_camera,
                    spawn_gradient_backdrop,
                    spawn_vignette,
                    spawn_dust_motes,
                ),
            )
            .add_systems(
                Update,
                (draw_breathing_grid, draw_floor_line, move_dust_motes),
            );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: WORLD_VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// §2.1: full-window vertical gradient, approximated as a stack of thin horizontal
/// bands (Bevy's 2D gizmos/sprites have no built-in gradient fill) — spawned once at
/// `Startup` since the gradient never animates.
fn spawn_gradient_backdrop(mut commands: Commands) {
    let band_height = (GRADIENT_HALF_EXTENT * 2.0) / GRADIENT_BAND_COUNT as f32;
    for i in 0..GRADIENT_BAND_COUNT {
        let t = i as f32 / (GRADIENT_BAND_COUNT - 1) as f32;
        let color = GRADIENT_TOP.lerp(GRADIENT_BOTTOM, t);
        let y = GRADIENT_HALF_EXTENT - (i as f32 + 0.5) * band_height;
        commands.spawn((
            Sprite {
                color: Color::srgb(color.x, color.y, color.z),
                custom_size: Some(Vec2::new(GRADIENT_HALF_EXTENT * 2.0, band_height * 1.05)),
                ..default()
            },
            Transform::from_xyz(0.0, y, BACKGROUND_Z),
        ));
    }
}

/// §2.2: vertical/horizontal grid lines with a pulsing alpha,
/// `alpha = 40 + 30·pulse`, `pulse = 0.5 + 0.5·sin(0.8·t)`.
fn draw_breathing_grid(mut gizmos: Gizmos, time: Res<Time>) {
    let t = time.elapsed_secs_f64();
    let pulse = 0.5 + 0.5 * (0.8 * t).sin();
    let alpha = ((40.0 + 30.0 * pulse) / 255.0) as f32;
    let color = Color::srgba(60.0 / 255.0, 80.0 / 255.0, 140.0 / 255.0, alpha);

    for &x in &GRID_VERTICAL_LINES {
        gizmos.line_2d(
            Vec2::new(x, -GRID_HALF_HEIGHT),
            Vec2::new(x, GRID_HALF_HEIGHT),
            color,
        );
    }
    for &y in &GRID_HORIZONTAL_LINES {
        gizmos.line_2d(
            Vec2::new(-GRID_HALF_WIDTH, y),
            Vec2::new(GRID_HALF_WIDTH, y),
            color,
        );
    }
}

/// §2.5: world floor reference line, (−3, −2) → (3, −2), (60, 70, 100, 220).
fn draw_floor_line(mut gizmos: Gizmos) {
    gizmos.line_2d(
        Vec2::new(-FLOOR_HALF_WIDTH, FLOOR_Y),
        Vec2::new(FLOOR_HALF_WIDTH, FLOOR_Y),
        Color::srgba(60.0 / 255.0, 70.0 / 255.0, 100.0 / 255.0, 220.0 / 255.0),
    );
}

/// §2.3: 80-px vignette bands on all four window edges, approximated as a small stack
/// of semi-transparent rects fading from edge-strength toward the center (Bevy UI 0.16
/// has no built-in gradient fill). Static — spawned once, sits under the HUD and dust
/// motes via a very negative `GlobalZIndex`.
const VIGNETTE_BAND_PX: f32 = 80.0;
const VIGNETTE_SUB_BANDS: u32 = 4;
/// Horizontal-edge (left/right) bands fade from (0,0,0,120); vertical (top/bottom) from
/// (0,0,0,100) — §2.3.
const VIGNETTE_HORIZONTAL_PEAK_ALPHA: f32 = 120.0 / 255.0;
const VIGNETTE_VERTICAL_PEAK_ALPHA: f32 = 100.0 / 255.0;

fn spawn_vignette(mut commands: Commands) {
    let sub_band_px = VIGNETTE_BAND_PX / VIGNETTE_SUB_BANDS as f32;
    for i in 0..VIGNETTE_SUB_BANDS {
        // Fades linearly from the window edge (strongest) toward the interior (transparent).
        let fraction = 1.0 - (i as f32 / VIGNETTE_SUB_BANDS as f32);
        let offset = i as f32 * sub_band_px;

        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(offset),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(sub_band_px),
                ..default()
            },
            BackgroundColor(Color::srgba(
                0.0,
                0.0,
                0.0,
                VIGNETTE_HORIZONTAL_PEAK_ALPHA * fraction,
            )),
            GlobalZIndex(-15),
        ));
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(offset),
                top: Val::Px(0.0),
                bottom: Val::Px(0.0),
                width: Val::Px(sub_band_px),
                ..default()
            },
            BackgroundColor(Color::srgba(
                0.0,
                0.0,
                0.0,
                VIGNETTE_HORIZONTAL_PEAK_ALPHA * fraction,
            )),
            GlobalZIndex(-15),
        ));
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(offset),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(sub_band_px),
                ..default()
            },
            BackgroundColor(Color::srgba(
                0.0,
                0.0,
                0.0,
                VIGNETTE_VERTICAL_PEAK_ALPHA * fraction,
            )),
            GlobalZIndex(-15),
        ));
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(offset),
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                height: Val::Px(sub_band_px),
                ..default()
            },
            BackgroundColor(Color::srgba(
                0.0,
                0.0,
                0.0,
                VIGNETTE_VERTICAL_PEAK_ALPHA * fraction,
            )),
            GlobalZIndex(-15),
        ));
    }
}

/// One cosmetic dust mote's deterministic base state (§2.4). Motion each frame is a pure
/// function of `t` plus these fields — the mote never reads or writes sim state.
#[derive(Component, Debug, Clone, Copy)]
struct DustMote {
    base_x: f32,
    base_y: f32,
    speed: f32,
    phase: f32,
}

/// §2.4: `h₀ = i·2654435761`, then LCG steps `h ← h·1664525 + 1013904223` between each
/// of the six draws (x, y, speed, phase, size, alpha) — mirrors the reference's
/// integer-hash sequence exactly so the same index always yields the same mote.
fn dust_mote_hash_sequence(index: u32) -> [u32; 6] {
    let mut h = index.wrapping_mul(DUST_HASH_SEED_MULTIPLIER);
    let mut draws = [0u32; 6];
    for draw in &mut draws {
        h = h
            .wrapping_mul(DUST_LCG_MULTIPLIER)
            .wrapping_add(DUST_LCG_INCREMENT);
        *draw = h;
    }
    draws
}

fn spawn_dust_motes(mut commands: Commands) {
    for i in 0..DUST_MOTE_COUNT {
        let [hx, hy, hspeed, hphase, hsize, halpha] = dust_mote_hash_sequence(i);
        let base_x = hx as f32 / u32::MAX as f32;
        let base_y = hy as f32 / u32::MAX as f32;
        let speed = 0.005 + (hspeed % 100) as f32 / 10000.0;
        let phase = (hphase % 628) as f32 / 100.0;
        let size = 1.0 + (hsize % 20) as f32 / 10.0;
        let alpha = (20 + halpha % 40) as f32 / 255.0;

        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(base_x * 100.0),
                top: Val::Percent(base_y * 100.0),
                width: Val::Px(size),
                height: Val::Px(size),
                ..default()
            },
            BackgroundColor(Color::srgba(180.0 / 255.0, 200.0 / 255.0, 1.0, alpha)),
            BorderRadius::all(Val::Percent(50.0)),
            GlobalZIndex(-20),
            DustMote {
                base_x,
                base_y,
                speed,
                phase,
            },
        ));
    }
}

/// §2.4 motion: upward drift `y − speed·t·0.1` wrapped to `[0, 1)`; sway
/// `x + 0.01·sin(0.5·t + phase)`.
fn move_dust_motes(time: Res<Time>, mut motes: Query<(&DustMote, &mut Node)>) {
    let t = time.elapsed_secs_f64() as f32;
    for (mote, mut node) in &mut motes {
        let y = (mote.base_y - mote.speed * t * 0.1).rem_euclid(1.0);
        let x = mote.base_x + 0.01 * (0.5 * t + mote.phase).sin();
        node.left = Val::Percent(x.rem_euclid(1.0) * 100.0);
        node.top = Val::Percent(y * 100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dust_mote_hash_is_deterministic_per_index() {
        assert_eq!(dust_mote_hash_sequence(0), dust_mote_hash_sequence(0));
        assert_ne!(dust_mote_hash_sequence(0), dust_mote_hash_sequence(1));
    }
}
