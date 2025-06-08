use colored::*;

pub fn log_offset(offset: usize, log_style: bool, action: &str) {
    if log_style {
        println!("{}", format!("[OFFSET] At: 0x{:X}", offset).cyan());
    } else {
        println!("{}", format!("{} at Offset: 0x{:X}", action, offset).cyan());
    }
}
pub fn log_pattern_found(pattern: &str, log_style: bool) {
    if log_style {
        println!(
            "{}",
            format!("[FOUND] Match for pattern: {}", pattern.blue()).green()
        );
    }
}

pub fn log_method_found(method_name: &str, offset: usize, log_style: bool) {
    if log_style {
        println!(
            "{}",
            format!("[FOUND] Method name: {}", method_name.blue()).green()
        );
    } else {
        println!(
            "{}",
            format!("Found {} at Offset: 0x{:X}", method_name, offset).green()
        );
    }
}

pub fn log_no_offset_found(method_name: &str, log_style: bool) {
    if log_style {
        println!(
            "{}",
            format!("[WARN] No offset found for {}.", method_name.yellow()).bold()
        );
    } else {
        println!(
            "{}",
            format!("Warning: No offset found for {}.", method_name).yellow()
        );
    }
}
pub fn log_patch_action(action: &str, hex: &str, log_style: bool) {
    if log_style {
        println!("{}", format!("[PATCH] {} with: {}", action, hex).purple());
    }
}

pub fn log_patch_skip(item: &str, reason: &str, log_style: bool) {
    if log_style {
        println!(
            "{}",
            format!(
                "[WARN] {}\nPatch failed: {}",
                reason.bright_yellow(),
                item.blue()
            )
            .bright_red()
            .bold()
        );
    } else {
        println!(
            "{}",
            format!(
                "Warning: {}\nPatch failed: {}",
                reason.bright_yellow(),
                item.blue()
            )
            .bright_red()
            .bold()
        );
    }
}

pub fn log_patch_error(item: &str, error: &str, log_style: bool) {
    if log_style {
        println!("{}", format!("[ERROR] {}: {}", item, error).red());
    } else {
        println!("{}", format!("Error: {}: {}", item, error).red());
    }
}

pub fn log_patch_done(output: &str, log_style: bool) {
    if log_style {
        println!("{}", format!("[DONE] File Save as: {}", output).green());
    } else {
        println!("{}", format!("File Save as: {}", output).green());
    }
}
