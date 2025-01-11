use bytemuck::cast;

pub fn format_hash(hash: i32) -> String {
    format!("{:0>8X}", cast::<i32, u32>(hash))
}
