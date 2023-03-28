use crate::db::spaces as spaces;

/// Capitalizes the first character in s.
pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Convert to space name with space replaced with a _
pub fn to_space_name(s: &str) -> String {
    return s.replace(" ", "_");
}


pub fn verify_space_exists(space: &str) -> bool {
    let space = spaces::get_space_by_name(space);
    if space.id == 0 {
        return false
    }
    return true
}