// Holds all the logic for the infinite precision numbers that the drawing will use
// Contains things like the operations struct, and the string numbers gen

pub mod imaginary {
    pub fn pow((real, imaginary): (f64, f64), power: f64) -> (f64, f64) {
        let theta = (imaginary / real).atan();
        let r = (real * real + imaginary * imaginary).powf(power / 2.);
        (r * (theta * power).cos(), r * (theta * power).sin())
    }
}
