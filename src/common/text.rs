use subtle::ConstantTimeEq;

const CHARACTER_SET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
pub fn get_random_letters(max: i32) -> String {
    (0..max)
        .map(|_| {
            let idx = rand::random_range(0..CHARACTER_SET.len());
            CHARACTER_SET[idx] as char
        })
        .collect()
}

const SECRET_CHARACTER_SET: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

pub fn get_random_secret(len: usize) -> String {
    (0..len)
        .map(|_| {
            let idx = rand::random_range(0..SECRET_CHARACTER_SET.len());
            SECRET_CHARACTER_SET[idx] as char
        })
        .collect()
}

pub fn is_constant_time_equals(a: &str, b: &str) -> bool {
    return a.as_bytes().ct_eq(b.as_bytes()).into();
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

    #[test]
    fn test_is_constant_time_equals_equal() {
        assert!(is_constant_time_equals("test", "test"));
    }
}
