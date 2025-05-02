use rand::Rng;

pub fn generate_otp() -> String {
    rand::thread_rng().gen_range(100000..999999).to_string()
}
