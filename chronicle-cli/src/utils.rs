use bytemuck::cast;
use indicatif::ProgressBar;

pub fn format_hash(hash: i32) -> String {
    format!("{:0>8X}", cast::<i32, u32>(hash))
}

pub fn limit_len(string: &str, len: usize) -> String {
    if string.len() <= len {
        return string.to_owned();
    }

    format!("{}...", &string[..len - 3])
}
