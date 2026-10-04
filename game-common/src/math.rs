//! 2D Math helpers skeleton.

use glam::Vec2;

/// Linearly interpolates between scalar `a` and `b` by factor `t`.
#[inline]
pub fn lerp(_a: f32, _b: f32, _t: f32) -> f32 {
    todo!("Calculate scalar lerp: a + (b - a) * t")
}

/// Linearly interpolates between `Vec2` `a` and `b` by factor `t`.
#[inline]
pub fn lerp_vec2(_a: Vec2, _b: Vec2, _t: f32) -> Vec2 {
    todo!("Calculate vector lerp: a + (b - a) * t")
}

/// Tests whether line segment `(p, p + d)` intersects circle at `centre` with `radius`.
/// Returns parametric distance `t ∈ [0, 1]` of first intersection if hit.
pub fn segment_circle_intersect(
    _p: Vec2,
    _d: Vec2,
    _centre: Vec2,
    _radius: f32,
) -> Option<f32> {
    todo!("Calculate quadratic discriminant for raycast intersection with circle")
}

#[inline]
pub fn segment_hits_circle(p: Vec2, d: Vec2, centre: Vec2, radius: f32) -> bool {
    segment_circle_intersect(p, d, centre, radius).is_some()
}
