use axum_test::TestServer;
use error_report_web_rs::{create_test_app, AppState};
use serde_json::json;

#[tokio::test]
async fn test_submit_error_report() {
    let server = TestServer::new(create_test_app().await).unwrap();

    let payload = json!({
        "machine": "qemux86-64",
        "distro": "poky",
        "distro_version": "4.0",
        "build_sys": "x86_64-linux",
        "nativelsbstring": "ubuntu-20.04",
        "target_sys": "x86_64-poky-linux",
        "failure_task": "do_compile",
        "failure_package": "example-package",
        "error_type": "CompilationError",
        "error_details": "compilation failed with error xyz",
        "log_data": "detailed log content here",
        "branch_commit": "abc123def456",
        "build_configuration": {
            "bb_version": "2.0.0",
            "tune_features": "m64 core2",
            "target_fpu": "",
            "meta_layers": []
        }
    });

    let response = server.post("/ClientPost/JSON/").json(&payload).await;

    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert!(body["id"].as_i64().unwrap() > 0);
    assert!(body["url"].as_str().unwrap().contains("Errors/Details"));
    assert_eq!(body["status"], "success");
}

#[tokio::test]
async fn test_list_errors_api() {
    let server = TestServer::new(create_test_app().await).unwrap();

    // First submit an error
    let payload = json!({
        "machine": "qemux86-64",
        "distro": "poky",
        "distro_version": "4.0",
        "build_sys": "x86_64-linux",
        "nativelsbstring": "ubuntu-20.04",
        "target_sys": "x86_64-poky-linux",
        "failure_task": "do_compile",
        "failure_package": "test-package",
        "error_type": "TestError",
        "error_details": "test error details",
        "log_data": "test log data",
        "branch_commit": "test123"
    });

    server
        .post("/ClientPost/JSON/")
        .json(&payload)
        .await
        .assert_status_ok();

    // Now test listing
    let response = server.get("/api/errors").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert!(body["errors"].as_array().unwrap().len() > 0);
    assert!(body["pagination"]["total"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn test_error_filtering() {
    let server = TestServer::new(create_test_app().await).unwrap();

    // Submit multiple errors with different machines
    let payload1 = json!({
        "machine": "qemux86-64",
        "distro": "poky",
        "distro_version": "4.0",
        "build_sys": "x86_64-linux",
        "nativelsbstring": "ubuntu-20.04",
        "target_sys": "x86_64-poky-linux",
        "failure_task": "do_compile",
        "failure_package": "package1",
        "error_type": "CompilationError",
        "error_details": "error 1",
        "log_data": "log 1",
        "branch_commit": "abc123"
    });

    let payload2 = json!({
        "machine": "qemuarm",
        "distro": "poky",
        "distro_version": "4.0",
        "build_sys": "x86_64-linux",
        "nativelsbstring": "ubuntu-20.04",
        "target_sys": "arm-poky-linux-gnueabi",
        "failure_task": "do_fetch",
        "failure_package": "package2",
        "error_type": "FetchError",
        "error_details": "error 2",
        "log_data": "log 2",
        "branch_commit": "def456"
    });

    server
        .post("/ClientPost/JSON/")
        .json(&payload1)
        .await
        .assert_status_ok();
    server
        .post("/ClientPost/JSON/")
        .json(&payload2)
        .await
        .assert_status_ok();

    // Test filtering by machine
    let response = server.get("/api/errors?machine=qemux86-64").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    let errors = body["errors"].as_array().unwrap();
    assert!(errors.len() >= 1);
    assert_eq!(errors[0]["machine"], "qemux86-64");
}

#[tokio::test]
async fn test_get_specific_error() {
    let server = TestServer::new(create_test_app().await).unwrap();

    // Submit an error
    let payload = json!({
        "machine": "qemux86-64",
        "distro": "poky",
        "distro_version": "4.0",
        "build_sys": "x86_64-linux",
        "nativelsbstring": "ubuntu-20.04",
        "target_sys": "x86_64-poky-linux",
        "failure_task": "do_compile",
        "failure_package": "specific-package",
        "error_type": "SpecificError",
        "error_details": "specific error details",
        "log_data": "specific log data",
        "branch_commit": "specific123"
    });

    let submit_response = server.post("/ClientPost/JSON/").json(&payload).await;
    submit_response.assert_status_ok();

    let submit_body: serde_json::Value = submit_response.json();
    let error_id = submit_body["id"].as_i64().unwrap();

    // Get the specific error
    let response = server.get(&format!("/api/errors/{}", error_id)).await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert_eq!(body["id"], error_id);
    assert_eq!(body["machine"], "qemux86-64");
    assert_eq!(body["failure_package"], "specific-package");
}

#[tokio::test]
async fn test_stats_endpoint() {
    let server = TestServer::new(create_test_app().await).unwrap();

    let response = server.get("/api/stats").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    assert!(body["total_errors"].as_u64().is_some());
    assert!(body["errors_by_type"].as_array().is_some());
    assert!(body["errors_by_machine"].as_array().is_some());
}

#[tokio::test]
async fn test_web_interface() {
    let server = TestServer::new(create_test_app().await).unwrap();

    // Test homepage
    let response = server.get("/").await;
    response.assert_status_ok();
    response.assert_text_contains("Yocto Project Error Reporting");

    // Test error list page
    let response = server.get("/Errors").await;
    response.assert_status_ok();
    response.assert_text_contains("Error Reports");

    // Test stats page
    let response = server.get("/Stats").await;
    response.assert_status_ok();
    response.assert_text_contains("Statistics");
}

#[tokio::test]
async fn test_invalid_error_submission() {
    let server = TestServer::new(create_test_app().await).unwrap();

    // Test with missing required fields
    let invalid_payload = json!({
        "machine": "qemux86-64",
        // missing required fields
    });

    let response = server
        .post("/ClientPost/JSON/")
        .json(&invalid_payload)
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_search_functionality() {
    let server = TestServer::new(create_test_app().await).unwrap();

    // Submit an error with searchable content
    let payload = json!({
        "machine": "qemux86-64",
        "distro": "poky",
        "distro_version": "4.0",
        "build_sys": "x86_64-linux",
        "nativelsbstring": "ubuntu-20.04",
        "target_sys": "x86_64-poky-linux",
        "failure_task": "do_compile",
        "failure_package": "searchable-package",
        "error_type": "SearchError",
        "error_details": "unique searchable error message",
        "log_data": "log data",
        "branch_commit": "search123"
    });

    server
        .post("/ClientPost/JSON/")
        .json(&payload)
        .await
        .assert_status_ok();

    // Search for the error
    let response = server.get("/api/errors?search=unique+searchable").await;
    response.assert_status_ok();

    let body: serde_json::Value = response.json();
    let errors = body["errors"].as_array().unwrap();
    assert!(errors.len() > 0);
    assert!(errors[0]["error_details"]
        .as_str()
        .unwrap()
        .contains("unique searchable"));
}
