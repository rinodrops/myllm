use std::io::{BufRead, BufReader, Read};

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde_json::{json, Value};

use crate::config::ResolvedRun;
use crate::error::{Error, Result};

pub fn stream_run(run: &ResolvedRun, on_token: impl FnMut(&str)) -> Result<()> {
    match run.provider.as_str() {
        "ollama" => stream_ollama(run, on_token),
        "openai" => stream_openai(run, on_token),
        "anthropic" => stream_anthropic(run, on_token),
        other => Err(Error::Provider(format!("unknown provider: {other}"))),
    }
}

fn http_client() -> Result<Client> {
    Ok(Client::builder()
        .build()
        .map_err(|err| Error::Http(err.to_string()))?)
}

fn stream_ollama(run: &ResolvedRun, on_token: impl FnMut(&str)) -> Result<()> {
    let url = format!("{}/api/chat", run.base_url.trim_end_matches('/'));
    let body = json!({
        "model": run.model,
        "messages": [{"role": "user", "content": run.prompt}],
        "stream": true,
        "keep_alive": run.keep_alive,
    });
    let response = http_client()?.post(url).json(&body).send()?;
    let status = response.status();
    if !status.is_success() {
        let text = response.text().unwrap_or_default();
        return Err(Error::Http(format!("Ollama HTTP {status}: {text}")));
    }
    parse_ollama_ndjson(response, on_token)
}

fn stream_openai(run: &ResolvedRun, on_token: impl FnMut(&str)) -> Result<()> {
    let key = run.api_key.as_deref().ok_or_else(|| {
        Error::Config(
            "OpenAI API key not found. Set api_key or api_key_env in the provider".into(),
        )
    })?;
    let url = format!("{}/chat/completions", run.base_url.trim_end_matches('/'));
    let body = json!({
        "model": run.model,
        "messages": [{"role": "user", "content": run.prompt}],
        "stream": true,
    });
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    let auth = HeaderValue::from_str(&format!("Bearer {key}"))
        .map_err(|err| Error::Config(err.to_string()))?;
    headers.insert(reqwest::header::AUTHORIZATION, auth);
    let response = http_client()?.post(url).headers(headers).json(&body).send()?;
    let status = response.status();
    if !status.is_success() {
        let text = response.text().unwrap_or_default();
        return Err(Error::Http(format!("OpenAI HTTP {status}: {text}")));
    }
    parse_openai_sse(response, on_token)
}

fn stream_anthropic(run: &ResolvedRun, on_token: impl FnMut(&str)) -> Result<()> {
    let key = run.api_key.as_deref().ok_or_else(|| {
        Error::Config(
            "Anthropic API key not found. Set api_key or api_key_env in the provider".into(),
        )
    })?;
    let url = format!("{}/messages", run.base_url.trim_end_matches('/'));
    let body = json!({
        "model": run.model,
        "max_tokens": 4096,
        "messages": [{"role": "user", "content": run.prompt}],
        "stream": true,
    });
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    headers.insert(
        "x-api-key",
        HeaderValue::from_str(key).map_err(|err| Error::Config(err.to_string()))?,
    );
    headers.insert(
        "anthropic-version",
        HeaderValue::from_static("2023-06-01"),
    );
    let response = http_client()?.post(url).headers(headers).json(&body).send()?;
    let status = response.status();
    if !status.is_success() {
        let text = response.text().unwrap_or_default();
        return Err(Error::Http(format!("Anthropic HTTP {status}: {text}")));
    }
    parse_anthropic_sse(response, on_token)
}

pub fn parse_ollama_ndjson(reader: impl Read, mut on_token: impl FnMut(&str)) -> Result<()> {
    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)
            .map_err(|err| Error::Provider(format!("invalid Ollama JSON: {err}")))?;
        if let Some(err) = value.get("error").and_then(Value::as_str) {
            if !err.is_empty() {
                return Err(Error::Provider(err.to_string()));
            }
        }
        if let Some(content) = value
            .pointer("/message/content")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            on_token(content);
        }
    }
    Ok(())
}

pub fn parse_openai_sse(reader: impl Read, mut on_token: impl FnMut(&str)) -> Result<()> {
    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let line = line?;
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let value: Value = serde_json::from_str(data)
            .map_err(|err| Error::Provider(format!("invalid OpenAI JSON: {err}")))?;
        if let Some(err) = value.pointer("/error/message").and_then(Value::as_str) {
            return Err(Error::Provider(err.to_string()));
        }
        if let Some(content) = value
            .pointer("/choices/0/delta/content")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
        {
            on_token(content);
        }
    }
    Ok(())
}

pub fn parse_anthropic_sse(reader: impl Read, mut on_token: impl FnMut(&str)) -> Result<()> {
    let reader = BufReader::new(reader);
    for line in reader.lines() {
        let line = line?;
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(data)
            .map_err(|err| Error::Provider(format!("invalid Anthropic JSON: {err}")))?;
        let kind = value.get("type").and_then(Value::as_str).unwrap_or("");
        if kind == "error" {
            let msg = value
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("Anthropic error");
            return Err(Error::Provider(msg.to_string()));
        }
        if kind == "content_block_delta" {
            if let Some(text) = value.pointer("/delta/text").and_then(Value::as_str) {
                if !text.is_empty() {
                    on_token(text);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn ollama_tokens_and_error() {
        let mut out = String::new();
        parse_ollama_ndjson(
            Cursor::new(
                r#"{"message":{"content":"Hel"}}
{"message":{"content":"lo"}}
{"done":true}
"#,
            ),
            |t| out.push_str(t),
        )
        .unwrap();
        assert_eq!(out, "Hello");

        let err = parse_ollama_ndjson(
            Cursor::new(r#"{"error":"model not found"}"#),
            |_| {},
        )
        .unwrap_err()
        .to_string();
        assert_eq!(err, "model not found");
    }

    #[test]
    fn openai_sse_tokens() {
        let mut out = String::new();
        parse_openai_sse(
            Cursor::new(
                "data: {\"choices\":[{\"delta\":{\"content\":\"Hi\"}}]}\n\
data: {\"choices\":[{\"delta\":{\"content\":\"!\"}}]}\n\
data: [DONE]\n",
            ),
            |t| out.push_str(t),
        )
        .unwrap();
        assert_eq!(out, "Hi!");
    }

    #[test]
    fn anthropic_sse_tokens() {
        let mut out = String::new();
        parse_anthropic_sse(
            Cursor::new(
                "event: content_block_delta\n\
data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"Hey\"}}\n\
data: {\"type\":\"message_stop\"}\n",
            ),
            |t| out.push_str(t),
        )
        .unwrap();
        assert_eq!(out, "Hey");
    }
}
