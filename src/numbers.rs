// Holds all the logic for the infinite precision numbers that the drawing will use
// Contains things like the operations struct, and the string numbers gen

pub mod imaginary {
    pub fn pow((real, imaginary): (f64, f64), power: f64) -> (f64, f64) {
        let mut theta = (imaginary / real).atan();
        if real < 0. || imaginary < 0. {
            theta += 180.;
        }
        let r = (real * real + imaginary * imaginary).powf(power / 2.);
        (r * (theta * power).cos(), r * (theta * power).sin())
    }

    pub fn square((real, imaginary): (f64, f64)) -> (f64, f64) {
        let (rr, ri) = {
            let r = real * real - imaginary * imaginary;
            let i = 2. * real * imaginary;
            (r, i)
        };
        (rr, ri)
    }
}
