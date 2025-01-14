use std::str::FromStr;

use chronicle::tag::DiscriminatedTag;

#[tauri::command]
pub fn parse_tag(tag: String) -> Result<DiscriminatedTag, chronicle::Error> {
    Ok(DiscriminatedTag::from_str(&tag)?)
}
