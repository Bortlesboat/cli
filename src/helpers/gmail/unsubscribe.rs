// Copyright 2026 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Gmail `+unsubscribe` helper -- RFC 8058 one-click list unsubscribe.

use super::*;
use std::collections::HashMap;

/// Handle the `+unsubscribe` subcommand.
pub async fn handle_unsubscribe(matches: &ArgMatches) -> Result<(), GwsError> {
    let max: u32 = matches
        .get_one::<String>("max")
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let list_mode = matches.get_flag("list");
    let from_filter = matches.get_one::<String>("from").map(|s| s.as_str());
    let dry_run = matches.get_flag("dry-run");
    let output_format = matches
        .get_one::<String>("format")
        .map(|s| crate::formatter::OutputFormat::from_str(s))
        .unwrap_or(crate::formatter::OutputFormat::Table);

    // Authenticate
    let token = auth::get_token(&[GMAIL_SCOPE], None)
        .await
        .map_err(|e| GwsError::Auth(format!("Gmail auth failed: {e}")))?;

    let client = crate::client::build_client()?;

    if list_mode {
        list_unsubscribe_candidates(&client, &token, max, &output_format).await
    } else if let Some(sender) = from_filter {
        unsubscribe_from_sender(&client, &token, sender, dry_run, max).await
    } else {
        Err(GwsError::Validation(
            "Specify --list to scan for candidates, or --from <sender> to unsubscribe".to_string(),
        ))
    }
}

/// Scan recent emails for List-Unsubscribe headers and group by sender.
async fn list_unsubscribe_candidates(
    client: &reqwest::Client,
    token: &str,
    max: u32,
    output_format: &crate::formatter::OutputFormat,
) -> Result<(), GwsError> {
    let messages = fetch_messages_with_unsubscribe_headers(client, token, max).await?;

    if messages.is_empty() {
        println!("No messages with List-Unsubscribe headers found.");
        return Ok(());
    }

    // Group by sender
    let mut by_sender: HashMap<String, SenderInfo> = HashMap::new();
    for msg in &messages {
        let entry = by_sender
            .entry(msg.from.clone())
            .or_insert_with(|| SenderInfo {
                from: msg.from.clone(),
                count: 0,
                has_rfc8058: false,
                unsubscribe_url: None,
                mailto: None,
            });
        entry.count += 1;
        if msg.has_post_header {
            entry.has_rfc8058 = true;
        }
        if entry.unsubscribe_url.is_none() {
            entry.unsubscribe_url.clone_from(&msg.https_url);
        }
        if entry.mailto.is_none() {
            entry.mailto.clone_from(&msg.mailto);
        }
    }

    let mut senders: Vec<SenderInfo> = by_sender.into_values().collect();
    senders.sort_by(|a, b| b.count.cmp(&a.count));

    let results: Vec<Value> = senders
        .iter()
        .map(|s| {
            let method = if s.has_rfc8058 {
                "RFC 8058 (one-click POST)"
            } else if s.mailto.is_some() {
                "mailto only"
            } else if s.unsubscribe_url.is_some() {
                "URL only (may need browser)"
            } else {
                "unknown"
            };
            json!({
                "from": s.from,
                "count": s.count,
                "method": method,
                "rfc8058": s.has_rfc8058,
            })
        })
        .collect();

    let output = json!({
        "candidates": results,
        "total_senders": senders.len(),
        "total_messages": messages.len(),
    });

    println!("{}", crate::formatter::format_value(&output, output_format));

    Ok(())
}

/// Unsubscribe from a specific sender using RFC 8058 one-click POST.
async fn unsubscribe_from_sender(
    client: &reqwest::Client,
    token: &str,
    sender: &str,
    dry_run: bool,
    max: u32,
) -> Result<(), GwsError> {
    let messages = fetch_messages_with_unsubscribe_headers(client, token, max).await?;

    // Find a message from the specified sender
    let matching: Vec<&MessageUnsubInfo> = messages
        .iter()
        .filter(|m| m.from.to_lowercase().contains(&sender.to_lowercase()))
        .collect();

    if matching.is_empty() {
        return Err(GwsError::Other(anyhow::anyhow!(
            "No messages with List-Unsubscribe header found from sender matching '{sender}'"
        )));
    }

    // Prefer one with RFC 8058 support
    let best = matching
        .iter()
        .find(|m| m.has_post_header && m.https_url.is_some())
        .or_else(|| matching.iter().find(|m| m.https_url.is_some()));

    let msg = match best {
        Some(m) => *m,
        None => {
            let mailto = matching.iter().find_map(|m| m.mailto.as_ref());
            if let Some(mailto_addr) = mailto {
                eprintln!("No HTTPS unsubscribe URL found for '{sender}'.");
                eprintln!("Mailto fallback available: {mailto_addr}");
                eprintln!(
                    "You can send an unsubscribe email with:\n  gws gmail +send --to '{mailto_addr}' --subject 'Unsubscribe' --body 'Unsubscribe'"
                );
                return Ok(());
            }
            return Err(GwsError::Other(anyhow::anyhow!(
                "No usable unsubscribe mechanism found for '{sender}'"
            )));
        }
    };

    let url = msg.https_url.as_ref().unwrap();

    if !msg.has_post_header {
        eprintln!("Warning: Sender does not advertise RFC 8058 List-Unsubscribe-Post header.");
        eprintln!("The POST may not work -- a browser visit may be required.");
        eprintln!("URL: {url}");
        if !dry_run {
            eprintln!("Attempting POST anyway...");
        }
    }

    if dry_run {
        eprintln!("[DRY RUN] Would POST to: {url}");
        eprintln!("[DRY RUN] Body: List-Unsubscribe=One-Click");
        eprintln!("[DRY RUN] Sender: {}", msg.from);
        eprintln!("[DRY RUN] Message ID: {}", msg.message_id);
        let output = json!({
            "action": "unsubscribe",
            "dry_run": true,
            "from": msg.from,
            "url": url,
            "method": if msg.has_post_header { "RFC 8058" } else { "HTTPS (no Post header)" },
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&output).unwrap_or_default()
        );
        return Ok(());
    }

    // Validate URL scheme before POSTing
    if !url.starts_with("https://") {
        return Err(GwsError::Validation(format!(
            "Refusing to POST to non-HTTPS URL: {url}"
        )));
    }

    // Perform RFC 8058 one-click unsubscribe POST
    eprintln!("Sending one-click unsubscribe POST to: {url}");
    let resp = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body("List-Unsubscribe=One-Click")
        .send()
        .await
        .map_err(|e| GwsError::Other(anyhow::anyhow!("Unsubscribe POST failed: {e}")))?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    let output = json!({
        "action": "unsubscribe",
        "from": msg.from,
        "url": url,
        "status": status.as_u16(),
        "success": status.is_success(),
        "response": if body.len() > 500 { body[..500].to_string() } else { body },
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&output).unwrap_or_default()
    );

    if status.is_success() {
        eprintln!("Unsubscribe successful for: {}", msg.from);
    } else {
        eprintln!(
            "Unsubscribe returned HTTP {status}. The sender may require a browser-based confirmation."
        );
    }

    Ok(())
}

/// Fetch messages with List-Unsubscribe headers from the Gmail API.
async fn fetch_messages_with_unsubscribe_headers(
    client: &reqwest::Client,
    token: &str,
    max: u32,
) -> Result<Vec<MessageUnsubInfo>, GwsError> {
    let list_url = "https://gmail.googleapis.com/gmail/v1/users/me/messages";

    let list_resp = client
        .get(list_url)
        .query(&[("q", "list:unsubscribe"), ("maxResults", &max.to_string())])
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| GwsError::Other(anyhow::anyhow!("Failed to list messages: {e}")))?;

    if !list_resp.status().is_success() {
        let err = list_resp.text().await.unwrap_or_default();
        return Err(GwsError::Api {
            code: 0,
            message: err,
            reason: "list_failed".to_string(),
            enable_url: None,
        });
    }

    let list_json: Value = list_resp
        .json()
        .await
        .map_err(|e| GwsError::Other(anyhow::anyhow!("Failed to parse list response: {e}")))?;

    let messages = match list_json.get("messages").and_then(|m| m.as_array()) {
        Some(m) => m,
        None => return Ok(Vec::new()),
    };

    // Fetch metadata for each message concurrently
    use futures_util::stream::{self, StreamExt};

    let msg_ids: Vec<String> = messages
        .iter()
        .filter_map(|m| m.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
        .collect();

    let results: Vec<MessageUnsubInfo> = stream::iter(msg_ids)
        .map(|msg_id| {
            let client = &client;
            let token = &token;
            async move {
                let get_url = format!(
                    "https://gmail.googleapis.com/gmail/v1/users/me/messages/{}?format=metadata\
                     &metadataHeaders=From\
                     &metadataHeaders=List-Unsubscribe\
                     &metadataHeaders=List-Unsubscribe-Post",
                    crate::validate::encode_path_segment(&msg_id),
                );

                let get_resp =
                    crate::client::send_with_retry(|| client.get(&get_url).bearer_auth(token))
                        .await
                        .ok()?;

                if !get_resp.status().is_success() {
                    return None;
                }

                let msg_json: Value = get_resp.json().await.ok()?;

                let headers = msg_json
                    .get("payload")
                    .and_then(|p| p.get("headers"))
                    .and_then(|h| h.as_array());

                let mut from = String::new();
                let mut list_unsub = String::new();
                let mut has_post_header = false;

                if let Some(headers) = headers {
                    for h in headers {
                        let name = h.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let value = h.get("value").and_then(|v| v.as_str()).unwrap_or("");
                        match name {
                            "From" => from = value.to_string(),
                            "List-Unsubscribe" => list_unsub = value.to_string(),
                            "List-Unsubscribe-Post" => {
                                has_post_header = value.contains("List-Unsubscribe=One-Click");
                            }
                            _ => {}
                        }
                    }
                }

                if list_unsub.is_empty() {
                    return None;
                }

                let (https_url, mailto) = parse_list_unsubscribe(&list_unsub);

                Some(MessageUnsubInfo {
                    message_id: msg_id,
                    from,
                    https_url,
                    mailto,
                    has_post_header,
                })
            }
        })
        .buffer_unordered(10)
        .filter_map(|r| async { r })
        .collect()
        .await;

    Ok(results)
}

/// Parse the List-Unsubscribe header value into HTTPS URL and mailto address.
///
/// The header format is a comma-separated list of angle-bracket-enclosed URIs:
///   `<https://example.com/unsub>, <mailto:unsub@example.com>`
fn parse_list_unsubscribe(header: &str) -> (Option<String>, Option<String>) {
    let mut https_url = None;
    let mut mailto = None;

    for part in header.split(',') {
        let trimmed = part.trim();
        // Extract content between < and >
        if let Some(start) = trimmed.find('<') {
            if let Some(end) = trimmed.find('>') {
                let uri = trimmed[start + 1..end].trim();
                if uri.starts_with("https://") && https_url.is_none() {
                    https_url = Some(uri.to_string());
                } else if uri.starts_with("mailto:") && mailto.is_none() {
                    mailto = Some(uri.strip_prefix("mailto:").unwrap_or(uri).to_string());
                }
            }
        }
    }

    (https_url, mailto)
}

#[derive(Debug)]
struct MessageUnsubInfo {
    message_id: String,
    from: String,
    https_url: Option<String>,
    mailto: Option<String>,
    has_post_header: bool,
}

struct SenderInfo {
    from: String,
    count: u32,
    has_rfc8058: bool,
    unsubscribe_url: Option<String>,
    mailto: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_list_unsubscribe_both() {
        let header = "<https://example.com/unsubscribe?id=123>, <mailto:unsub@example.com>";
        let (https, mailto) = parse_list_unsubscribe(header);
        assert_eq!(https.unwrap(), "https://example.com/unsubscribe?id=123");
        assert_eq!(mailto.unwrap(), "unsub@example.com");
    }

    #[test]
    fn test_parse_list_unsubscribe_https_only() {
        let header = "<https://example.com/unsub>";
        let (https, mailto) = parse_list_unsubscribe(header);
        assert_eq!(https.unwrap(), "https://example.com/unsub");
        assert!(mailto.is_none());
    }

    #[test]
    fn test_parse_list_unsubscribe_mailto_only() {
        let header = "<mailto:leave@example.com>";
        let (https, mailto) = parse_list_unsubscribe(header);
        assert!(https.is_none());
        assert_eq!(mailto.unwrap(), "leave@example.com");
    }

    #[test]
    fn test_parse_list_unsubscribe_empty() {
        let (https, mailto) = parse_list_unsubscribe("");
        assert!(https.is_none());
        assert!(mailto.is_none());
    }

    #[test]
    fn test_parse_list_unsubscribe_http_rejected() {
        // Only HTTPS URLs should be extracted, not plain HTTP
        let header = "<http://example.com/unsub>";
        let (https, mailto) = parse_list_unsubscribe(header);
        assert!(https.is_none());
        assert!(mailto.is_none());
    }

    #[test]
    fn test_parse_list_unsubscribe_multiple_urls() {
        let header = "<https://first.com/unsub>, <https://second.com/unsub>, <mailto:a@b.com>";
        let (https, mailto) = parse_list_unsubscribe(header);
        // Should pick the first HTTPS URL
        assert_eq!(https.unwrap(), "https://first.com/unsub");
        assert_eq!(mailto.unwrap(), "a@b.com");
    }

    #[test]
    fn test_parse_list_unsubscribe_whitespace() {
        let header = "  < https://example.com/unsub > , < mailto:a@b.com > ";
        let (https, mailto) = parse_list_unsubscribe(header);
        assert_eq!(https.unwrap(), "https://example.com/unsub");
        assert_eq!(mailto.unwrap(), "a@b.com");
    }
}
