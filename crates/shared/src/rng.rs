// https://www.shadertoy.com/view/4djSRW
// Hash without Sine
// MIT License...
/* Copyright (c)2014 David Hoskins.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.*/

use crate::*;


//----------------------------------------------------------------------------------------
//  1 out, 1 in...
pub fn hash11(mut p: f32) -> f32 {
    p = <f32 as Real>::fract(p * 0.1031f32);
    p *= p + 33.33f32;
    p *= p + p;
    <f32 as Real>::fract(p)
}

//----------------------------------------------------------------------------------------
//  1 out, 2 in...
pub fn hash12(p: Vec2) -> f32 {
	let mut p3 = Vec3::fract(p.xyx() * 0.1031f32);
    p3 += Vec3::dot(p3, p3.yzx() + 33.33f32);
    <f32 as Real>::fract((p3.x + p3.y) * p3.z)
}

//----------------------------------------------------------------------------------------
//  1 out, 3 in...
pub fn hash13(p: Vec3) -> f32 {
	let mut p3  = Vec3::fract(p * 0.1031f32);
    p3 += Vec3::dot(p3, p3.zyx() + 31.32f32);
    <f32 as Real>::fract((p3.x + p3.y) * p3.z)
}

//----------------------------------------------------------------------------------------
///  3 out, 3 in...
pub fn hash33(mut p3: Vec3) -> Vec3 {
	p3 = Vec3::fract(p3 * vec3(0.1031, 0.1030, 0.0973));
    p3 += Vec3::dot(p3, p3.yxz()+33.33);
    return Vec3::fract((p3.xxy() + p3.yxx())*p3.zyx());

}