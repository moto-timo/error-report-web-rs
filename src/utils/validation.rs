use crate::models::error_report::ErrorSubmissionData;

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Field '{field}' is required")]
    RequiredField { field: String },
    #[error("Field '{field}' is too long (max: {max}, actual: {actual})")]
    TooLong { field: String, max: usize, actual: usize },
    #[error("Field '{field}' contains invalid characters")]
    InvalidCharacters { field: String },
    #[error("Invalid email format")]
    InvalidEmail,
}

pub fn validate_error_submission(data: &ErrorSubmissionData) -> Result<(), ValidationError> {
    // Required fields validation
    validate_required_field(&data.machine, "machine")?;
    validate_required_field(&data.distro, "distro")?;
    validate_required_field(&data.distro_version, "distro_version")?;
    validate_required_field(&data.build_sys, "build_sys")?;
    validate_required_field(&data.nativelsbstring, "nativelsbstring")?;
    validate_required_field(&data.target_sys, "target_sys")?;
    validate_required_field(&data.failure_task, "failure_task")?;
    validate_required_field(&data.failure_package, "failure_package")?;
    validate_required_field(&data.error_type, "error_type")?;
    validate_required_field(&data.error_details, "error_details")?;
    validate_required_field(&data.log_data, "log_data")?;
    validate_required_field(&data.branch_commit, "branch_commit")?;

    // Length validation
    validate_max_length(&data.machine, "machine", 100)?;
    validate_max_length(&data.distro, "distro", 100)?;
    validate_max_length(&data.distro_version, "distro_version", 50)?;
    validate_max_length(&data.build_sys, "build_sys", 100)?;
    validate_max_length(&data.nativelsbstring, "nativelsbstring", 100)?;
    validate_max_length(&data.target_sys, "target_sys", 100)?;
    validate_max_length(&data.failure_task, "failure_task", 200)?;
    validate_max_length(&data.failure_package, "failure_package", 200)?;
    validate_max_length(&data.error_type, "error_type", 100)?;
    validate_max_length(&data.branch_commit, "branch_commit", 100)?;

    // Optional field validation
    if let Some(name) = &data.submitter_name {
        validate_max_length(name, "submitter_name", 100)?;
    }

    if let Some(email) = &data.submitter_email {
        validate_max_length(email, "submitter_email", 200)?;
        validate_email(email)?;
    }

    // Content validation
    validate_no_harmful_content(&data.error_details)?;
    validate_no_harmful_content(&data.log_data)?;

    Ok(())
}

fn validate_required_field(value: &str, field_name: &str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        return Err(ValidationError::RequiredField {
            field: field_name.to_string(),
        });
    }
    Ok(())
}

fn validate_max_length(value: &str, field_name: &str, max_length: usize) -> Result<(), ValidationError> {
    if value.len() > max_length {
        return Err(ValidationError::TooLong {
            field: field_name.to_string(),
            max: max_length,
            actual: value.len(),
        });
    }
    Ok(())
}

fn validate_email(email: &str) -> Result<(), ValidationError> {
    // Simple email validation - for production, consider using a proper email validation crate
    if !email.contains('@') || !email.contains('.') {
        return Err(ValidationError::InvalidEmail);
    }
    Ok(())
}

fn validate_no_harmful_content(content: &str) -> Result<(), ValidationError> {
    // Basic check for potentially harmful content
    let harmful_patterns = [
        "<script", "</script>", "javascript:", "vbscript:", "onload=", "onerror="
    ];

    let content_lower = content.to_lowercase();
    for pattern in &harmful_patterns {
        if content_lower.contains(pattern) {
            return Err(ValidationError::InvalidCharacters {
                field: "content".to_string(),
            });
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::error_report::ErrorSubmissionData;

    fn create_valid_submission() -> ErrorSubmissionData {
        ErrorSubmissionData {
            machine: "qemux86-64".to_string(),
            distro: "poky".to_string(),
            distro_version: "4.0".to_string(),
            build_sys: "x86_64-linux".to_string(),
            nativelsbstring: "ubuntu-20.04".to_string(),
            target_sys: "x86_64-poky-linux".to_string(),
            failure_task: "do_compile".to_string(),
            failure_package: "example-package".to_string(),
            error_type: "CompilationError".to_string(),
            error_details: "compilation failed".to_string(),
            log_data: "detailed log content".to_string(),
            submitter_name: Some("Test User".to_string()),
            submitter_email: Some("test@example.com".to_string()),
            branch_commit: "abc123def456".to_string(),
            build_configuration: None,
        }
    }

    #[test]
    fn test_valid_submission() {
        let submission = create_valid_submission();
        assert!(validate_error_submission(&submission).is_ok());
    }

    #[test]
    fn test_empty_required_field() {
        let mut submission = create_valid_submission();
        submission.machine = "".to_string();
        assert!(validate_error_submission(&submission).is_err());
    }

    #[test]
    fn test_too_long_field() {
        let mut submission = create_valid_submission();
        submission.machine = "a".repeat(200);
        assert!(validate_error_submission(&submission).is_err());
    }

    #[test]
    fn test_invalid_email() {
        let mut submission = create_valid_submission();
        submission.submitter_email = Some("invalid-email".to_string());
        assert!(validate_error_submission(&submission).is_err());
    }

    #[test]
    fn test_harmful_content() {
        let mut submission = create_valid_submission();
        submission.error_details = "<script>alert('xss')</script>".to_string();
        assert!(validate_error_submission(&submission).is_err());
    }
}
