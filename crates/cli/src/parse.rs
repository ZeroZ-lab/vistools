use std::path::Path;

use vistools_core::{CommandResult, ErrorCode, ErrorInfo, Rect, SampleOutput};

pub fn parse_u32_arg(name: &str, value: &str) -> Result<u32, String> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|_| format!("{name} must be an unsigned integer"))
}

pub fn parse_rect_arg(value: &str) -> Result<Rect, String> {
    let parts: Vec<_> = value.split(',').map(str::trim).collect();
    if parts.len() != 4 || parts.iter().any(|part| part.is_empty()) {
        return Err("rect must use x,y,width,height syntax".to_string());
    }

    let x = parse_u32_arg("rect.x", parts[0])?;
    let y = parse_u32_arg("rect.y", parts[1])?;
    let width = parse_u32_arg("rect.width", parts[2])?;
    let height = parse_u32_arg("rect.height", parts[3])?;

    Ok(Rect {
        x,
        y,
        width,
        height,
    })
}

pub fn parse_optional_rect_arg(value: Option<String>) -> Result<Option<Rect>, String> {
    value.as_deref().map(parse_rect_arg).transpose()
}

pub fn invalid_sample_parameters(input: &Path, message: impl Into<String>) -> (String, bool) {
    let result = CommandResult::<SampleOutput>::err(
        "sample",
        input.display().to_string(),
        ErrorInfo::with_message(ErrorCode::InvalidParameters, message),
    );
    (serde_json::to_string_pretty(&result).unwrap(), false)
}

pub fn invalid_region_parameters<T: serde::Serialize>(
    operation: &str,
    input: &Path,
    message: impl Into<String>,
) -> (String, bool) {
    let result = CommandResult::<T>::err(
        operation,
        input.display().to_string(),
        ErrorInfo::with_message(ErrorCode::InvalidParameters, message),
    );
    (serde_json::to_string_pretty(&result).unwrap(), false)
}
