use rand::Rng;

const CHARACTER_SET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
pub fn get_random_letters(max: i32) -> String {
    let mut rng = rand::thread_rng();

    (0..max)
        .map(|_| {
            let idx = rng.gen_range(0..CHARACTER_SET.len());
            CHARACTER_SET[idx] as char
        })
        .collect()
}
