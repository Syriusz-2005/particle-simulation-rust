use graphics::math::Vec2d;

#[inline(always)]
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
pub fn div_scalar(v: &mut Vec2d, a: f64) -> &mut Vec2d {
    v[0] /= a;
    v[1] /= a;
    return v;
}

#[inline(always)]
pub fn mul_hadamard<'a>(v1: &'a mut Vec2d, v2: &Vec2d) -> &'a mut Vec2d {
    v1[0] *= v2[0];
    v1[1] *= v2[1];
    return v1;
}

#[inline(always)]
pub fn remap(x: f64, a: f64, b: f64, c: f64, d: f64) -> f64 {
    return c + (x - a) * (d - c) / (b - a);
}
