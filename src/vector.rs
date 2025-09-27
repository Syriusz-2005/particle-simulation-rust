use core::arch::x86_64;
use graphics::math::Vec2d;
use rand::{distr::uniform::SampleUniform, rngs::ThreadRng, Rng};
use std::ops::Range;

use crate::constants::K;

#[inline(always)]
#[allow(dead_code)]
pub fn random_vec<T: SampleUniform + Clone + PartialOrd>(
    rng: &mut ThreadRng,
    r: Range<T>,
) -> Vec2d<T> {
    [rng.random_range(r.clone()), rng.random_range(r.clone())]
}

#[inline(always)]
#[allow(dead_code)]
pub fn add_scalar(v: &mut Vec2d, a: f64) -> &mut Vec2d {
    v[0] += a;
    v[1] += a;
    return v;
}

#[inline(always)]
pub fn add<'a>(v1: &'a mut Vec2d, v2: &Vec2d) -> &'a mut Vec2d {
    v1[0] += v2[0];
    v1[1] += v2[1];
    return v1;
}

#[inline(always)]
#[allow(dead_code)]
pub fn add_simd<'a>(v1: &'a mut Vec2d, v2: &Vec2d) -> &'a mut Vec2d {
    unsafe {
        let dest_ptr = v1.as_mut_ptr();
        let source_ptr = v2.as_ptr();
        let v2_packed = x86_64::_mm_loadu_pd(source_ptr);
        let v1_packed = x86_64::_mm_loadu_pd(dest_ptr);
        let output = x86_64::_mm_add_pd(v1_packed, v2_packed);
        x86_64::_mm_store_pd(dest_ptr, output);
    }
    return v1;
}

#[inline(always)]
#[allow(dead_code)]
pub fn sub_scalar(v1: &mut Vec2d, a: f64) -> &mut Vec2d {
    v1[0] -= a;
    v1[1] -= a;
    return v1;
}

#[inline(always)]
pub fn sub<'a>(v1: &'a mut Vec2d, v2: &Vec2d) -> &'a mut Vec2d {
    v1[0] -= v2[0];
    v1[1] -= v2[1];
    return v1;
}

#[inline(always)]
#[allow(dead_code)]
pub fn sub_simd<'a>(v1: &'a mut Vec2d, v2: &Vec2d) -> &'a mut Vec2d {
    unsafe {
        let dest_ptr = v1.as_mut_ptr();
        let source_ptr = v2.as_ptr();
        let v2_packed = x86_64::_mm_loadu_pd(source_ptr);
        let v1_packed = x86_64::_mm_loadu_pd(dest_ptr);
        let output = x86_64::_mm_sub_pd(v1_packed, v2_packed);
        x86_64::_mm_store_pd(dest_ptr, output);
    }
    return v1;
}

#[inline(always)]
pub fn len(v: &Vec2d) -> f64 {
    return (v[0].powi(2) + v[1].powi(2)).sqrt();
}

#[inline(always)]
pub fn normalize(v: &mut Vec2d) -> &mut Vec2d {
    let length = len(v);
    v[0] /= length;
    v[1] /= length;
    return v;
}

#[inline(always)]
pub fn mul_scalar(v: &mut Vec2d, a: f64) -> &mut Vec2d {
    v[0] *= a;
    v[1] *= a;
    return v;
}

#[inline(always)]
#[allow(dead_code)]
pub fn mul_scalar_simd(v: &mut Vec2d, a: f64) -> &mut Vec2d {
    unsafe {
        let dest_ptr = v.as_mut_ptr();
        let val_packed = x86_64::_mm_set1_pd(a);
        let v1_packed = x86_64::_mm_loadu_pd(dest_ptr);
        let output = x86_64::_mm_mul_pd(v1_packed, val_packed);
        x86_64::_mm_store_pd(dest_ptr, output);
    }
    return v;
}

#[inline(always)]
#[allow(dead_code)]
pub fn apply_forces(
    mut total_force: &mut Vec2d,
    direction: &Vec2d,
    force_value: f64,
    distance_based_strength: f64,
) {
    let force = &mut direction.clone();
    unsafe {
        let dest_ptr = force.as_mut_ptr();
        let val_packed = x86_64::_mm_set1_pd(force_value);
        let force_packed = x86_64::_mm_loadu_pd(dest_ptr);
        let result = x86_64::_mm_mul_pd(force_packed, val_packed);
        let dist_strength_packed = x86_64::_mm_set1_pd(distance_based_strength);
        let result = x86_64::_mm_mul_pd(result, dist_strength_packed);
        let k_packed = x86_64::_mm_set1_pd(K);
        let force_multiplied = x86_64::_mm_mul_pd(result, k_packed);
        x86_64::_mm_store_pd(dest_ptr, force_multiplied);
    }
    add(&mut total_force, force);
}

#[inline(always)]
pub fn div_scalar(v: &mut Vec2d, a: f64) -> &mut Vec2d {
    v[0] /= a;
    v[1] /= a;
    return v;
}

#[inline(always)]
#[allow(dead_code)]
pub fn mul_hadamard<'a>(v1: &'a mut Vec2d, v2: &Vec2d) -> &'a mut Vec2d {
    v1[0] *= v2[0];
    v1[1] *= v2[1];
    return v1;
}

#[inline(always)]
pub fn remap(x: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
    return c + (x - a) * (d - c) / (b - a);
}
