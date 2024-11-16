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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_random_letters_length() {
        let result = get_random_letters(10);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn test_get_random_letters_content() {
        let result = get_random_letters(10);
        for c in result.chars() {
            assert!(CHARACTER_SET.contains(&(c as u8)));
        }
    }

    #[test]
    fn test_get_random_letters_zero_length() {
        let result = get_random_letters(0);
        assert_eq!(result.len(), 0);
    }
}

