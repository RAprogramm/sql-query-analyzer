// SPDX-FileCopyrightText: 2025 RAprogramm
// SPDX-License-Identifier: MIT

use std::{
    fs::write,
    io::{Read, Write},
    thread::spawn
};

use sql_query_analyzer::{
    app::{AnalyzeParams, run_analyze},
    cli::{Dialect, Format, Provider},
    config::Config
};

fn mock_ollama_server(body: &'static str) -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    spawn(move || {
        if let Ok((mut sock, _)) = listener.accept() {
            let mut buf = [0u8; 8192];
            let _ = sock.read(&mut buf);
            let resp = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = sock.write_all(resp.as_bytes());
        }
    });
    format!("http://{addr}")
}

#[tokio::test]
async fn run_analyze_with_llm_appends_analysis() {
    let server = mock_ollama_server(r#"{"response":"llm analysis text"}"#);
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("schema.sql");
    let queries = dir.path().join("queries.sql");
    write(&schema, "CREATE TABLE users (id INT PRIMARY KEY);").unwrap();
    write(&queries, "SELECT * FROM users WHERE id = 1;").unwrap();
    let params = AnalyzeParams {
        schema_path:   schema.to_string_lossy().into_owned(),
        queries_path:  queries.to_string_lossy().into_owned(),
        provider:      Provider::Ollama,
        api_key:       None,
        model:         None,
        ollama_url:    server,
        dialect:       Dialect::Generic,
        output_format: Format::Text,
        verbose:       false,
        dry_run:       false,
        no_color:      true
    };
    let result = run_analyze(params, Config::default()).await.unwrap();
    let llm_output = result.llm_output.expect("llm output present");
    assert!(llm_output.contains("llm analysis text"), "{llm_output}");
}

#[tokio::test]
async fn run_analyze_llm_failure_propagates_error() {
    let dir = tempfile::tempdir().unwrap();
    let schema = dir.path().join("schema.sql");
    let queries = dir.path().join("queries.sql");
    write(&schema, "CREATE TABLE users (id INT PRIMARY KEY);").unwrap();
    write(&queries, "SELECT * FROM users;").unwrap();
    let params = AnalyzeParams {
        schema_path:   schema.to_string_lossy().into_owned(),
        queries_path:  queries.to_string_lossy().into_owned(),
        provider:      Provider::Ollama,
        api_key:       None,
        model:         None,
        ollama_url:    "http://127.0.0.1:1".into(),
        dialect:       Dialect::Generic,
        output_format: Format::Text,
        verbose:       false,
        dry_run:       false,
        no_color:      true
    };
    assert!(run_analyze(params, Config::default()).await.is_err());
}
