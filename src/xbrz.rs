// This upscales pixel art smoothly.
// Port of xBRZ pixel scaling algorithm to Rust.
// Based on code from: https://sourceforge.net/projects/xbrz/
// Port by Chris Hulbert 2018

use std::cmp;

const LUMINANCE_WEIGHT: f32             = 1.0;
const EQUAL_COLOR_TOLERANCE: f32        = 30.0;
const DOMINANT_DIRECTION_THRESHOLD: f32 = 3.6;
const STEEP_DIRECTION_THRESHOLD: f32    = 2.2;

// BlendType must fit into the value range of 2 bit!!!
// I'm using constants instead of an enum here because enums don't implement Copy which causes issues if you want to reuse them.
type BlendType = u8;
const BLEND_TYPE_NONE:     u8 = 0;
const BLEND_TYPE_NORMAL:   u8 = 1; //a normal indication to blend
const BLEND_TYPE_DOMINANT: u8 = 2; //a strong indication to blend

// clock-wise
#[repr(u8)] // Keep it small.
enum RotationDegree {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}

struct BlendResult {
    blend_f: BlendType,
    blend_g: BlendType,
    blend_j: BlendType,
    blend_k: BlendType,
}

struct Kernel4x4 { // kernel for preprocessing step
    a: u32,
    b: u32,
    c: u32, 
    _d: u32,
    e: u32,
    f: u32,
    g: u32,
    h: u32,
    i: u32,
    j: u32,
    k: u32,
    l: u32,
    _m: u32,
    n: u32,
    o: u32,
    _p: u32,
}

struct Kernel3x3 {
    a: u32,
    b: u32,
    c: u32, 
    d: u32,
    e: u32,
    f: u32,
    g: u32,
    h: u32,
    i: u32,
}

trait ColoursExt {
    fn r(&self) -> u8;
    fn g(&self) -> u8;
    fn b(&self) -> u8;
    fn a(&self) -> u8;
}

impl ColoursExt for u32 {
    #[inline]
    fn r(&self) -> u8 {
        (self>>24) as u8 // 'as' truncates/ignores larger numbers so no '& 0xff' required.
    }
    #[inline]
    fn g(&self) -> u8 {
        (self>>16) as u8
    }
    #[inline]
    fn b(&self) -> u8 {
        (self>>8) as u8
    }
    #[inline]
    fn a(&self) -> u8 {
        *self as u8
    }
}

#[inline]
fn make_pixel(r: u8, g: u8, b: u8, a: u8) -> u32 {
    return ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

#[inline]
fn dist(pix1: u32, pix2: u32) -> f32 {
    let a1: f32 = (pix1.a() as f32) / 255.0;
    let a2: f32 = (pix2.a() as f32) / 255.0;
    let d: f32 = dist_y_cb_cr(pix1, pix2);
    if a1 < a2 {
        return a1 * d + 255.0 * (a2 - a1);
    } else {
        return a2 * d + 255.0 * (a1 - a2);
    }
}

fn dist_y_cb_cr(pix1: u32, pix2: u32) -> f32 {
    //http://en.wikipedia.org/wiki/YCbCr#ITU-R_BT.601_conversion
    //YCbCr conversion is a matrix multiplication => take advantage of linearity by subtracting first!
    let r_diff: f32 = ((pix1.r() as i16) - (pix2.r() as i16)) as f32; //we may delay division by 255 to after matrix multiplication
    let g_diff: f32 = ((pix1.g() as i16) - (pix2.g() as i16)) as f32;
    let b_diff: f32 = ((pix1.b() as i16) - (pix2.b() as i16)) as f32; //substraction for int is noticeable faster than for f32!

    //const f32 K_B = 0.0722; //ITU-R BT.709 conversion
    //const f32 k_r = 0.2126; //
    const K_B: f32 = 0.0593; //ITU-R BT.2020 conversion
    const K_R: f32 = 0.2627; //
    const K_G: f32 = 1.0 - K_B - K_R;

    let scale_b: f32 = 0.5 / (1.0 - K_B);
    let scale_r: f32 = 0.5 / (1.0 - K_R);

    let y: f32 = K_R * r_diff + K_G * g_diff + K_B * b_diff; //[!], analog YCbCr!
    let c_b: f32 = scale_b * (b_diff - y);
    let c_r: f32 = scale_r * (r_diff - y);

    //we skip division by 255 to have similar range like other distance functions
    let l_y = LUMINANCE_WEIGHT * y;
    return (l_y*l_y + c_b*c_b + c_r*c_r).sqrt();
}

macro_rules! set_top_l { ($b:expr, $bt:expr) => { $b |= $bt } }
macro_rules! set_top_r { ($b:expr, $bt:expr) => { $b |= $bt << 2 } }
macro_rules! set_bottom_r { ($b:expr, $bt:expr) => { $b |= $bt << 4 } }
macro_rules! set_bottom_l { ($b:expr, $bt:expr) => { $b |= $bt << 6 } }

// Buffer is assumed to be initialized before preprocessing!
#[inline]
fn get_top_r(b: u8)    -> BlendType { return unsafe { ::std::mem::transmute(0x3 & (b >> 2)) }; }
#[inline]
fn get_bottom_r(b: u8) -> BlendType { return unsafe { ::std::mem::transmute(0x3 & (b >> 4)) }; }
#[inline]
fn get_bottom_l(b: u8) -> BlendType { return unsafe { ::std::mem::transmute(0x3 & (b >> 6)) }; }

// TODO replace these rotations with macros like in the C++ version?
impl Kernel3x3 {
    #[inline]
    fn rot90(&self) -> Kernel3x3 {
        return Kernel3x3 {
            a: self.g, b: self.d, c: self.a,
            d: self.h, e: self.e, f: self.b,
            g: self.i, h: self.f, i: self.c};
    }

    #[inline]
    fn rot180(&self) -> Kernel3x3 {
        return Kernel3x3 {
            a: self.i, b: self.h, c: self.g,
            d: self.f, e: self.e, f: self.d,
            g: self.c, h: self.b, i: self.a};
    }

    #[inline]
    fn rot270(&self) -> Kernel3x3 {
        return Kernel3x3 {
            a: self.c, b: self.f, c: self.i,
            d: self.b, e: self.e, f: self.h,
            g: self.a, h: self.d, i: self.g };
    }
}

// TODO replace these rotations with macros?
trait RotatableBlendInfo {
    fn blend_info_rot90(&self) -> u8;
    fn blend_info_rot180(&self) -> u8;
    fn blend_info_rot270(&self) -> u8;
}
impl RotatableBlendInfo for u8 {
    #[inline]
    fn blend_info_rot90(&self) -> u8 {
        return (self << 2) | (self >> 6)
    }

    #[inline]
    fn blend_info_rot180(&self) -> u8 {
        return (self << 4) | (self >> 4)
    }

    #[inline]
    fn blend_info_rot270(&self) -> u8 {
        return (self << 6) | (self >> 2)
    }
}

// pitch_elements is number of u32 elements, eg pixels, not bytes.
fn fill_block(trg: *mut u32, pitch_elements: isize, col: u32, block_width: isize, block_height: isize) {
    let mut offset: isize = 0;
    let next_line_offset: isize = pitch_elements - block_width;
    for _ in 0..block_height {
        for _ in 0..block_width {
            unsafe { *trg.offset(offset) = col; }
            offset += 1;
        }
        offset += next_line_offset;
    }
}

// find intermediate color between two colors with alpha channels (=> NO alpha blending!!!)
fn gradient_argb(m: i32, n: i32, pix_front: u32, pix_back: u32) -> u32 {
    let weight_front: i32 = (pix_front.a() as i32) * m;
    let weight_back: i32  = (pix_back.a() as i32) * (n - m);
    let weight_sum: i32   = weight_front + weight_back;
    if weight_sum == 0 {
        return 0;
    }

    let calc_color = |col_front: u8, col_back: u8| -> u8 {
        return (((col_front as i32)*weight_front + (col_back as i32)*weight_back) / weight_sum) as u8;
    };

    return make_pixel(
        calc_color(pix_front.r(), pix_back.r()),
        calc_color(pix_front.g(), pix_back.g()),
        calc_color(pix_front.b(), pix_back.b()),
        (weight_sum / n) as u8);
}

fn alpha_grad(m: i32, n: i32, pix_back: &mut u32, pix_front: u32) {
    *pix_back = gradient_argb(m, n, pix_front, *pix_back)
}

trait Scaler {
    fn scale(&self) -> u8;
    fn blend_line_shallow(&self, col: u32, out: &OutputMatrix);
    fn blend_line_steep(&self, col: u32, out: &OutputMatrix);
    fn blend_line_steep_and_shallow(&self, col: u32, out: &OutputMatrix);
    fn blend_line_diagonal(&self, col: u32, out: &OutputMatrix);
    fn blend_corner(&self, col: u32, out: &OutputMatrix);
}

struct Scaler2x {}
struct Scaler3x {}
struct Scaler4x {}
struct Scaler5x {}
struct Scaler6x {}

impl Scaler for Scaler2x {
    fn scale(&self) -> u8 {
        return 2;
    }

    fn blend_line_shallow(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(2 - 1, 0), col);
        alpha_grad(3, 4, out.pixel_ref(2 - 1, 1), col);
    }

    fn blend_line_steep(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(0, 2-1), col);
        alpha_grad(3, 4, out.pixel_ref(1, 2-1), col);
    }

    fn blend_line_steep_and_shallow(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(1, 0), col);
        alpha_grad(1, 4, out.pixel_ref(0, 1), col);
        alpha_grad(5, 6, out.pixel_ref(1, 1), col); //[!] fixes 7/8 used in xBR
    }

    fn blend_line_diagonal(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 2, out.pixel_ref(1, 1), col);
    }

    fn blend_corner(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(21, 100, out.pixel_ref(1, 1), col); //exact: 1 - pi/4 = 0.2146018366
    }
}

impl Scaler for Scaler3x {
    fn scale(&self) -> u8 {
        return 3;
    }

    fn blend_line_shallow(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(3 - 1, 0), col);
        alpha_grad(1, 4, out.pixel_ref(3 - 2, 2), col);

        alpha_grad(3, 4, out.pixel_ref(3 - 1, 1), col);
        *out.pixel_ref(3 - 1, 2) = col;
    }

    fn blend_line_steep(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(0, 3 - 1), col);
        alpha_grad(1, 4, out.pixel_ref(2, 3 - 2), col);

        alpha_grad(3, 4, out.pixel_ref(1, 3 - 1), col);
        *out.pixel_ref(2, 3 - 1) = col;
    }

    fn blend_line_steep_and_shallow(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(2, 0), col);
        alpha_grad(1, 4, out.pixel_ref(0, 2), col);
        alpha_grad(3, 4, out.pixel_ref(2, 1), col);
        alpha_grad(3, 4, out.pixel_ref(1, 2), col);
        *out.pixel_ref(2, 2) = col;
    }

    fn blend_line_diagonal(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 8, out.pixel_ref(1, 2), col); //conflict with other rotations for this odd scale
        alpha_grad(1, 8, out.pixel_ref(2, 1), col);
        alpha_grad(7, 8, out.pixel_ref(2, 2), col); //
    }

    fn blend_corner(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(45, 100, out.pixel_ref(2, 2), col); //exact: 0.4545939598
    }
}

impl Scaler for Scaler4x {
    fn scale(&self) -> u8 {
        return 4;
    }

    fn blend_line_shallow(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(4 - 1, 0), col);
        alpha_grad(1, 4, out.pixel_ref(4 - 2, 2), col);

        alpha_grad(3, 4, out.pixel_ref(4 - 1, 1), col);
        alpha_grad(3, 4, out.pixel_ref(4 - 2, 3), col);

        *out.pixel_ref(4 - 1, 2) = col;
        *out.pixel_ref(4 - 1, 3) = col;
    }

    fn blend_line_steep(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(0, 4 - 1), col);
        alpha_grad(1, 4, out.pixel_ref(2, 4 - 2), col);

        alpha_grad(3, 4, out.pixel_ref(1, 4 - 1), col);
        alpha_grad(3, 4, out.pixel_ref(3, 4 - 2), col);

        *out.pixel_ref(2, 4 - 1) = col;
        *out.pixel_ref(3, 4 - 1) = col;
    }

    fn blend_line_steep_and_shallow(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(3, 4, out.pixel_ref(3, 1), col);
        alpha_grad(3, 4, out.pixel_ref(1, 3), col);
        alpha_grad(1, 4, out.pixel_ref(3, 0), col);
        alpha_grad(1, 4, out.pixel_ref(0, 3), col);

        alpha_grad(1, 3, out.pixel_ref(2, 2), col); //[!] fixes 1/4 used in xBR

        *out.pixel_ref(3, 3) = col;
        *out.pixel_ref(3, 2) = col;
        *out.pixel_ref(2, 3) = col;
    }

    fn blend_line_diagonal(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 2, out.pixel_ref(4 - 1, 4 / 2    ), col);
        alpha_grad(1, 2, out.pixel_ref(4 - 2, 4 / 2 + 1), col);
        *out.pixel_ref(4 - 1, 4 - 1) = col;
    }

    fn blend_corner(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(68, 100, out.pixel_ref(3, 3), col); //exact: 0.6848532563
        alpha_grad( 9, 100, out.pixel_ref(3, 2), col); //0.08677704501
        alpha_grad( 9, 100, out.pixel_ref(2, 3), col); //0.08677704501
    }
}

impl Scaler for Scaler5x {
    fn scale(&self) -> u8 {
        return 5;
    }

    fn blend_line_shallow(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(5 - 1, 0), col);
        alpha_grad(1, 4, out.pixel_ref(5 - 2, 2), col);
        alpha_grad(1, 4, out.pixel_ref(5 - 3, 4), col);

        alpha_grad(3, 4, out.pixel_ref(5 - 1, 1), col);
        alpha_grad(3, 4, out.pixel_ref(5 - 2, 3), col);

        *out.pixel_ref(5 - 1, 2) = col;
        *out.pixel_ref(5 - 1, 3) = col;
        *out.pixel_ref(5 - 1, 4) = col;
        *out.pixel_ref(5 - 2, 4) = col;
    }

    fn blend_line_steep(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(0, 5 - 1), col);
        alpha_grad(1, 4, out.pixel_ref(2, 5 - 2), col);
        alpha_grad(1, 4, out.pixel_ref(4, 5 - 3), col);

        alpha_grad(3, 4, out.pixel_ref(1, 5 - 1), col);
        alpha_grad(3, 4, out.pixel_ref(3, 5 - 2), col);

        *out.pixel_ref(2, 5 - 1) = col;
        *out.pixel_ref(3, 5 - 1) = col;
        *out.pixel_ref(4, 5 - 1) = col;
        *out.pixel_ref(4, 5 - 2) = col;
    }

    fn blend_line_steep_and_shallow(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(0, 5 - 1), col);
        alpha_grad(1, 4, out.pixel_ref(2, 5 - 2), col);
        alpha_grad(3, 4, out.pixel_ref(1, 5 - 1), col);

        alpha_grad(1, 4, out.pixel_ref(5 - 1, 0), col);
        alpha_grad(1, 4, out.pixel_ref(5 - 2, 2), col);
        alpha_grad(3, 4, out.pixel_ref(5 - 1, 1), col);

        alpha_grad(2, 3, out.pixel_ref(3, 3), col);

        *out.pixel_ref(2, 5 - 1) = col;
        *out.pixel_ref(3, 5 - 1) = col;
        *out.pixel_ref(4, 5 - 1) = col;

        *out.pixel_ref(5 - 1, 2) = col;
        *out.pixel_ref(5 - 1, 3) = col;
    }

    fn blend_line_diagonal(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 8, out.pixel_ref(5 - 1, 5 / 2    ), col); //conflict with other rotations for this odd scale
        alpha_grad(1, 8, out.pixel_ref(5 - 2, 5 / 2 + 1), col);
        alpha_grad(1, 8, out.pixel_ref(5 - 3, 5 / 2 + 2), col); //

        alpha_grad(7, 8, out.pixel_ref(4, 3), col);
        alpha_grad(7, 8, out.pixel_ref(3, 4), col);

        *out.pixel_ref(4, 4) = col;
    }

    fn blend_corner(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(86, 100, out.pixel_ref(4, 4), col); //exact: 0.8631434088
        alpha_grad(23, 100, out.pixel_ref(4, 3), col); //0.2306749731
        alpha_grad(23, 100, out.pixel_ref(3, 4), col); //0.2306749731
    }
}

impl Scaler for Scaler6x {
    fn scale(&self) -> u8 {
        return 6;
    }

    fn blend_line_shallow(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(6 - 1, 0), col);
        alpha_grad(1, 4, out.pixel_ref(6 - 2, 2), col);
        alpha_grad(1, 4, out.pixel_ref(6 - 3, 4), col);

        alpha_grad(3, 4, out.pixel_ref(6 - 1, 1), col);
        alpha_grad(3, 4, out.pixel_ref(6 - 2, 3), col);
        alpha_grad(3, 4, out.pixel_ref(6 - 3, 5), col);

        *out.pixel_ref(6 - 1, 2) = col;
        *out.pixel_ref(6 - 1, 3) = col;
        *out.pixel_ref(6 - 1, 4) = col;
        *out.pixel_ref(6 - 1, 5) = col;

        *out.pixel_ref(6 - 2, 4) = col;
        *out.pixel_ref(6 - 2, 5) = col;
    }

    fn blend_line_steep(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(0, 6 - 1), col);
        alpha_grad(1, 4, out.pixel_ref(2, 6 - 2), col);
        alpha_grad(1, 4, out.pixel_ref(4, 6 - 3), col);

        alpha_grad(3, 4, out.pixel_ref(1, 6 - 1), col);
        alpha_grad(3, 4, out.pixel_ref(3, 6 - 2), col);
        alpha_grad(3, 4, out.pixel_ref(5, 6 - 3), col);

        *out.pixel_ref(2, 6 - 1) = col;
        *out.pixel_ref(3, 6 - 1) = col;
        *out.pixel_ref(4, 6 - 1) = col;
        *out.pixel_ref(5, 6 - 1) = col;

        *out.pixel_ref(4, 6 - 2) = col;
        *out.pixel_ref(5, 6 - 2) = col;
    }

    fn blend_line_steep_and_shallow(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 4, out.pixel_ref(0, 6 - 1), col);
        alpha_grad(1, 4, out.pixel_ref(2, 6 - 2), col);
        alpha_grad(3, 4, out.pixel_ref(1, 6 - 1), col);
        alpha_grad(3, 4, out.pixel_ref(3, 6 - 2), col);

        alpha_grad(1, 4, out.pixel_ref(6 - 1, 0), col);
        alpha_grad(1, 4, out.pixel_ref(6 - 2, 2), col);
        alpha_grad(3, 4, out.pixel_ref(6 - 1, 1), col);
        alpha_grad(3, 4, out.pixel_ref(6 - 2, 3), col);

        *out.pixel_ref(2, 6 - 1) = col;
        *out.pixel_ref(3, 6 - 1) = col;
        *out.pixel_ref(4, 6 - 1) = col;
        *out.pixel_ref(5, 6 - 1) = col;

        *out.pixel_ref(4, 6 - 2) = col;
        *out.pixel_ref(5, 6 - 2) = col;

        *out.pixel_ref(6 - 1, 2) = col;
        *out.pixel_ref(6 - 1, 3) = col;
    }

    fn blend_line_diagonal(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(1, 2, out.pixel_ref(6 - 1, 6 / 2    ), col);
        alpha_grad(1, 2, out.pixel_ref(6 - 2, 6 / 2 + 1), col);
        alpha_grad(1, 2, out.pixel_ref(6 - 3, 6 / 2 + 2), col);

        *out.pixel_ref(6 - 2, 6 - 1) = col;
        *out.pixel_ref(6 - 1, 6 - 1) = col;
        *out.pixel_ref(6 - 1, 6 - 2) = col;
    }

    fn blend_corner(&self, col: u32, out: &OutputMatrix) {
        alpha_grad(97, 100, out.pixel_ref(5, 5), col); //exact: 0.9711013910
        alpha_grad(42, 100, out.pixel_ref(4, 5), col); //0.4236372243
        alpha_grad(42, 100, out.pixel_ref(5, 4), col); //0.4236372243
        alpha_grad( 6, 100, out.pixel_ref(5, 3), col); //0.05652034508
        alpha_grad( 6, 100, out.pixel_ref(3, 5), col); //0.05652034508
    }
}

// Scale, degree template <size_t N, RotationDegree rot_deg>
struct OutputMatrix {
    scale: u8,
    rot_deg: RotationDegree,
    out: *mut u32,
    out_width: i32,
}

impl OutputMatrix {
    fn pixel_ref(&self, i: i32, j: i32) -> &mut u32 {
        let i_old = rotation_i_old(&self.rot_deg, i, j, self.scale);
        let j_old = rotation_j_old(&self.rot_deg, i, j, self.scale);
        return unsafe { self.out.offset((j_old + i_old * self.out_width) as isize).as_mut().unwrap() }
    } 
}

#[inline]
fn rotation_i_old(rot: &RotationDegree, i: i32, j: i32, n: u8) -> i32 {
    match rot {
        RotationDegree::Rot0 => i,
        RotationDegree::Rot90 => n as i32 - 1 - j,
        RotationDegree::Rot180 => n as i32 - 1 - i,
        RotationDegree::Rot270 => j,
    }
} 

#[inline]
fn rotation_j_old(rot: &RotationDegree, i: i32, j: i32, n: u8) -> i32 {
    match rot {
        RotationDegree::Rot0 => j,
        RotationDegree::Rot90 => i,
        RotationDegree::Rot180 => n as i32 - 1 - j,
        RotationDegree::Rot270 => n as i32 - 1 - i,
    }
}

/// result: F, G, J, K corners of "GradientType"
fn pre_process_corners(ker: &Kernel4x4) -> BlendResult {
    if (ker.f == ker.g && ker.j == ker.k) || (ker.f == ker.j && ker.g == ker.k) {
        return BlendResult { blend_f: BLEND_TYPE_NONE, blend_g: BLEND_TYPE_NONE, blend_j: BLEND_TYPE_NONE, blend_k: BLEND_TYPE_NONE };
    }

    const WEIGHT: f32 = 4.0;
    let jg: f32 = dist(ker.i, ker.f) + dist(ker.f, ker.c) + dist(ker.n, ker.k) + dist(ker.k, ker.h) + WEIGHT * dist(ker.j, ker.g);
    let fk: f32 = dist(ker.e, ker.j) + dist(ker.j, ker.o) + dist(ker.b, ker.g) + dist(ker.g, ker.l) + WEIGHT * dist(ker.f, ker.k);

    let mut blend_f = BLEND_TYPE_NONE;
    let mut blend_g = BLEND_TYPE_NONE;
    let mut blend_j = BLEND_TYPE_NONE;
    let mut blend_k = BLEND_TYPE_NONE;
    if jg < fk { // test sample: 70% of values cmp:max(jg, fk) / cmp::min(jg, fk) are between 1.1 and 3.7 with median being 1.8
        let dominant_gradient: BlendType = if DOMINANT_DIRECTION_THRESHOLD * jg < fk { BLEND_TYPE_DOMINANT } else { BLEND_TYPE_NORMAL };
        if ker.f != ker.g && ker.f != ker.j {
            blend_f = dominant_gradient;
        }
        if ker.k != ker.j && ker.k != ker.g {
            blend_k = dominant_gradient;
        }
    } else if fk < jg {
        let dominant_gradient: BlendType = if DOMINANT_DIRECTION_THRESHOLD * fk < jg { BLEND_TYPE_DOMINANT } else { BLEND_TYPE_NORMAL };
        if ker.j != ker.f && ker.j != ker.k {
            blend_j = dominant_gradient;
        }
        if ker.g != ker.f && ker.g != ker.k {
            blend_g = dominant_gradient;
        }
    }
    return BlendResult { blend_f: blend_f, blend_g: blend_g, blend_j: blend_j, blend_k: blend_k }
}

/// TODO use some kind of compile-time generics/templates instead.
fn select_scaler(scale: u8) -> Box<dyn Scaler> {
    if scale == 2 {
        return Box::new(Scaler2x {});
    } else if scale == 3 {
        return Box::new(Scaler3x {});
    } else if scale == 4 {
        return Box::new(Scaler4x {});
    } else if scale == 5 {
        return Box::new(Scaler5x {});
    } else if scale == 6 {
        return Box::new(Scaler6x {});
    } else {
        panic!("Invalid scale")
    }
}

fn blend_pixel(scale: u8,
                scaler: &Box<dyn Scaler>,
                rot_deg: RotationDegree,
                ker: &Kernel3x3,
                target: *mut u32,
                trg_width: u32,
                blend: u8) { //result of preprocessing all four corners of pixel "e"

    if get_bottom_r(blend) >= BLEND_TYPE_NORMAL {
        let eq = |pix1: u32, pix2: u32| -> bool {
            return dist(pix1, pix2) < EQUAL_COLOR_TOLERANCE;
        };

        let do_line_blend = (|| -> bool {
            if get_bottom_r(blend) >= BLEND_TYPE_DOMINANT {
                return true;
            }

            //make sure there is no second blending in an adjacent rotation for this pixel: handles insular pixels, mario eyes
            if get_top_r(blend) != BLEND_TYPE_NONE && !eq(ker.e, ker.g) { //but support double-blending for 90� corners
                return false;
            }
            if get_bottom_l(blend) != BLEND_TYPE_NONE && !eq(ker.e, ker.c) {
                return false;
            }

            //no full blending for L-shapes; blend corner only (handles "mario mushroom eyes")
            if !eq(ker.e, ker.i) && eq(ker.g, ker.h) && eq(ker.h, ker.i) && eq(ker.i, ker.f) && eq(ker.f, ker.c) {
                return false;
            }

            return true;
        })();

        let px: u32 = if dist(ker.e, ker.f) <= dist(ker.e, ker.h) { ker.f } else { ker.h }; //choose most similar color

        let out = &OutputMatrix { scale: scale, rot_deg: rot_deg, out: target, out_width: trg_width as i32 };

        if do_line_blend {
            let fg: f32 = dist(ker.f, ker.g); //test sample: 70% of values cmp:max(fg, hc) / cmp::min(fg, hc) are between 1.1 and 3.7 with median being 1.9
            let hc: f32 = dist(ker.h, ker.c); //

            let have_shallow_line: bool = STEEP_DIRECTION_THRESHOLD * fg <= hc && ker.e != ker.g && ker.d != ker.g;
            let have_steep_line: bool   = STEEP_DIRECTION_THRESHOLD * hc <= fg && ker.e != ker.c && ker.b != ker.c;

            if have_shallow_line {
                if have_steep_line {
                    scaler.blend_line_steep_and_shallow(px, out);
                } else {
                    scaler.blend_line_shallow(px, out);
                }
            } else {
                if have_steep_line {
                    scaler.blend_line_steep(px, out);
                } else {
                    scaler.blend_line_diagonal(px, out);
                }
            }
        } else {
            scaler.blend_corner(px, out);
        }
    }
}

fn do_scale(scale: u8, src: &[u32], trg: *mut u32, src_width: i32, src_height: i32) {
    let y_first: i32 = 0;
    let y_last = src_height;
    if y_first >= y_last { return }
    if src_width <= 0 { return }

    let trg_width = src_width * scale as i32;

    //"use" space at the end of the image as temporary buffer for "on the fly preprocessing": we even could use larger area of
    //"sizeof(uint32_t) * src_width * (y_last - y_first)" bytes without risk of accidental overwriting before accessing
    let buffer_size = src_width;
    let pre_proc_buffer_ptr: *mut u8 = unsafe { (trg.offset((y_last * scale as i32 * trg_width) as isize) as *mut u8).offset(-(buffer_size as isize)) };
    unsafe { pre_proc_buffer_ptr.write_bytes(0, buffer_size as usize) };
    let pre_proc_buffer: &mut [u8] = unsafe { std::slice::from_raw_parts_mut(pre_proc_buffer_ptr, buffer_size as usize) };
    // let mut pre_proc_buffer: Vec<u8> = vec![0; buffer_size as usize];

    let scaler = select_scaler(scale);

    //initialize preprocessing buffer for first row of current stripe: detect upper left and right corner blending
    //this cannot be optimized for adjacent processing stripes; we must not allow for a memory race condition!
    if y_first > 0 {
        let y = y_first - 1;

        let s_m1 = &src[((src_width * cmp::max(y - 1, 0)) as usize)..];
        let s_0  = &src[((src_width * y) as usize)..]; //center line
        let s_p1 = &src[((src_width * cmp::min(y + 1, src_height - 1)) as usize)..];
        let s_p2 = &src[((src_width * cmp::min(y + 2, src_height - 1)) as usize)..];

        for x in 0..src_width {
            let x_m1 = cmp::max(x - 1, 0) as usize;
            let x_p1 = cmp::min(x + 1, src_width - 1) as usize;
            let x_p2 = cmp::min(x + 2, src_width - 1) as usize;

            let ker = Kernel4x4 {
                a: s_m1[x_m1], //read sequentially from memory as far as possible
                b: s_m1[x as usize],
                c: s_m1[x_p1],
                _d: s_m1[x_p2],

                e: s_0[x_m1],
                f: s_0[x as usize],
                g: s_0[x_p1],
                h: s_0[x_p2],

                i: s_p1[x_m1],
                j: s_p1[x as usize],
                k: s_p1[x_p1],
                l: s_p1[x_p2],

                _m: s_p2[x_m1],
                n: s_p2[x as usize],
                o: s_p2[x_p1],
                _p: s_p2[x_p2] };

            let res = pre_process_corners(&ker);
            /*
            preprocessing blend result:
            ---------
            | F | G |   //evalute corner between F, G, J, K
            ----|---|   //input pixel is at position F
            | J | K |
            ---------
            */            
            set_top_r!(pre_proc_buffer[x as usize], res.blend_j);

            if x + 1 < buffer_size {
                set_top_l!(pre_proc_buffer[x as usize + 1], res.blend_k);
            }
        }
    }
    //------------------------------------------------------------------------------------
    for y in y_first..y_last {
        let mut out: *mut u32 = unsafe { trg.offset((scale as i32 * y * trg_width) as isize) };

        let s_m1 = &src[((src_width * cmp::max(y - 1, 0)) as usize)..];
        let s_0  = &src[((src_width * y) as usize)..]; //center line
        let s_p1 = &src[((src_width * cmp::min(y + 1, src_height - 1)) as usize)..];
        let s_p2 = &src[((src_width * cmp::min(y + 2, src_height - 1)) as usize)..];

        let mut blend_xy1: u8 = 0; //corner blending for current (x, y + 1) position

        for x in 0..src_width {
            //all those bounds checks have only insignificant impact on performance!
            let x_m1 = cmp::max(x as i32 - 1, 0) as usize; //perf: prefer array indexing to additional pointers!
            let x_p1 = cmp::min(x + 1, src_width - 1) as usize;
            let x_p2 = cmp::min(x + 2, src_width - 1) as usize;

            let ker4 = Kernel4x4 {
                a: s_m1[x_m1], //read sequentially from memory as far as possible
                b: s_m1[x as usize],
                c: s_m1[x_p1],
                _d: s_m1[x_p2],

                e: s_0[x_m1],
                f: s_0[x as usize],
                g: s_0[x_p1],
                h: s_0[x_p2],

                i: s_p1[x_m1],
                j: s_p1[x as usize],
                k: s_p1[x_p1],
                l: s_p1[x_p2],

                _m: s_p2[x_m1],
                n: s_p2[x as usize],
                o: s_p2[x_p1],
                _p: s_p2[x_p2]
            };

            //evaluate the four corners on bottom-right of current pixel
            let res = pre_process_corners(&ker4);
            /*
            preprocessing blend result:
            ---------
            | F | G |   //evalute corner between F, G, J, K
            ----|---|   //current input pixel is at position F
            | J | K |
            ---------
            */
            let mut blend_xy: u8 = pre_proc_buffer[x as usize]; //for current (x, y) position
            set_bottom_r!(blend_xy, res.blend_f); //all four corners of (x, y) have been determined at this point due to processing sequence!

            set_top_r!(blend_xy1, res.blend_j); //set 2nd known corner for (x, y + 1)
            pre_proc_buffer[x as usize] = blend_xy1; //store on current buffer position for use on next row

            blend_xy1 = 0;
            set_top_l!(blend_xy1, res.blend_k); //set 1st known corner for (x + 1, y + 1) and buffer for use on next column

            if x + 1 < buffer_size { //set 3rd known corner for (x + 1, y)
                set_bottom_l!(pre_proc_buffer[x as usize + 1], res.blend_g);
            }

            //fill block of size scale * scale with the given color            
            fill_block(out, trg_width as isize, ker4.f, scale as isize, scale as isize);
            //place *after* preprocessing step, to not overwrite the results while processing the the last pixel!

            //blend four corners of current pixel
            if blend_xy != 0 { //good 5% perf-improvement
                let ker3 = Kernel3x3 {
                    a: ker4.a,
                    b: ker4.b,
                    c: ker4.c,

                    d: ker4.e,
                    e: ker4.f,
                    f: ker4.g,

                    g: ker4.i,
                    h: ker4.j,
                    i: ker4.k };

                blend_pixel(scale, &scaler, RotationDegree::Rot0,   &ker3,          out, trg_width as u32, blend_xy);
                blend_pixel(scale, &scaler, RotationDegree::Rot90,  &ker3.rot90(),  out, trg_width as u32, blend_xy.blend_info_rot90());
                blend_pixel(scale, &scaler, RotationDegree::Rot180, &ker3.rot180(), out, trg_width as u32, blend_xy.blend_info_rot180());
                blend_pixel(scale, &scaler, RotationDegree::Rot270, &ker3.rot270(), out, trg_width as u32, blend_xy.blend_info_rot270());
            }

            unsafe { out = out.offset(scale as isize); }
        }
    }
}

pub fn scale(factor: u8, src: &[u32], src_width: u32, src_height: u32) -> Vec<u32> {
    if factor == 1 {
        return Vec::from(src);
    }

    let mut output: Vec<u32> = vec![0; (src_width * src_height * factor as u32 * factor as u32) as usize];
    do_scale(factor, src, output.as_mut_ptr(), src_width as i32, src_height as i32);
    return output;
}

// fn u8_to_u32_slice(original: &[u8]) -> &[u32] {
//     let count = original.len() / mem::size_of::<u32>();
//     let ptr = original.as_ptr() as *const u32;
//     return unsafe { slice::from_raw_parts(ptr, count) }; // Warning: potential alignment crash?
// }

// fn u32_to_u8_slice(original: &[u32]) -> &[u8] {
//     let count = original.len() * mem::size_of::<u32>();
//     let ptr = original.as_ptr() as *const u8;
//     return unsafe { slice::from_raw_parts(ptr, count) };
// }

// // Test with: cargo run --release 12ms
// fn main() {

//     let dynamic_img = image::open("lemmings.png").unwrap();
//     let img: RgbaImage = dynamic_img.to_rgba();
//     let scale_factor: u8 = 6;
//     let (width, height) = img.dimensions();
//     let data: Vec<u8> = img.into_raw();
//     let data_pixels = u8_to_u32_slice(&data);
//     // let data_pixels_rgba = png_to_scaler_format(&data_pixels);
//     let start = Instant::now();
//     let big: Vec<u32> = scale(scale_factor, data_pixels, width, height);
//     let elapsed = start.elapsed();
//     println!("Took {:?}", elapsed); // 170.6ms debug, 10ms optim
//     //let big_abgr = scaler_to_png_format(&big);
//     let big_buf = u32_to_u8_slice(&big);
//     image::save_buffer("big.png", big_buf, width * scale_factor as u32, height * scale_factor as u32, image::RGBA(8)).unwrap();

//     // x16 for 5k
//     let fourx = scale(4, data_pixels, width, height);
//     let sixteenx = scale(4, &fourx, width*4, height*4);
//     let sixteen_buf = u32_to_u8_slice(&sixteenx);
//     image::save_buffer("yuge.png", sixteen_buf, width * 16, height * 16, image::RGBA(8)).unwrap();

//     // The color method returns the image's `ColorType`.
//     // println!("{:?}", img.color());

//     // Write the contents of this image to the Writer in PNG format.
//     // img.save("test.png").unwrap();

//     let z: u32 = 0xccbbaa99;
//     let r = z.r();
//     let g = z.g();
//     let b = z.b();
//     let a = z.a();
//     println!("r: 0x{:x}", r);
//     println!("g: 0x{:x}", g);
//     println!("b: 0x{:x}", b);
//     println!("a: 0x{:x}", a);

//     // Make something
//     // let mut buffer: [u32; 4] = [0xff7777ff, 0x77ff77ff, 0x7777ffff, 0xffff77ff];
//     let mut buffer: [u32; 4] = [0; 4];
//     buffer[0] = make_pixel(0xff, 0xff, 0, 0);
//     buffer[0] = 0xff0000ff; // Different order to make_pixel.
//     buffer[1] = 0xffff0000; //ABGR
//     buffer[2] = 0xff00ff00;
//     buffer[3] = 0xff0000ff;
//     //let slice = &buffer;
//     // let _ptr = buffer.as_ptr() as *const u8;
//     let ptr = buffer.as_ptr() as *const u8;
//     // let size = mem::size_of::<[u32; 4]>();
//     let size = mem::size_of_val(&buffer);
//     let u8_slice = unsafe { slice::from_raw_parts(ptr, size) };

// // let buffer: &[u8] = ...; // Generate the image data

// //     // Save the buffer as "image.png"
//      image::save_buffer("output.png", u8_slice, 2, 2, image::RGBA(8)).unwrap()

// }
