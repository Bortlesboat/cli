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

use crate::error::GwsError;

/// A known service with its alias, API name, version, and description.
pub struct ServiceEntry {
    pub aliases: &'static [&'static str],
    pub api_name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

/// All known services with metadata.
pub const SERVICES: &[ServiceEntry] = &[
    ServiceEntry {
        aliases: &["drive"],
        api_name: "drive",
        version: "v3",
        description: "Manage files, folders, and shared drives",
    },
    ServiceEntry {
        aliases: &["sheets"],
        api_name: "sheets",
        version: "v4",
        description: "Read and write spreadsheets",
    },
    ServiceEntry {
        aliases: &["gmail"],
        api_name: "gmail",
        version: "v1",
        description: "Send, read, and manage email",
    },
    ServiceEntry {
        aliases: &["calendar"],
        api_name: "calendar",
        version: "v3",
        description: "Manage calendars and events",
    },
    ServiceEntry {
        aliases: &["admin", "directory"],
        api_name: "admin",
        version: "directory_v1",
        description: "Manage users, groups, and devices",
    },
    ServiceEntry {
        aliases: &["admin-reports", "reports"],
        api_name: "admin",
        version: "reports_v1",
        description: "Audit logs and usage reports",
    },
    ServiceEntry {
        aliases: &["docs"],
        api_name: "docs",
        version: "v1",
        description: "Read and write Google Docs",
    },
    ServiceEntry {
        aliases: &["slides"],
        api_name: "slides",
        version: "v1",
        description: "Read and write presentations",
    },
    ServiceEntry {
        aliases: &["tasks"],
        api_name: "tasks",
        version: "v1",
        description: "Manage task lists and tasks",
    },
    ServiceEntry {
        aliases: &["people"],
        api_name: "people",
        version: "v1",
        description: "Manage contacts and profiles",
    },
    ServiceEntry {
        aliases: &["chat"],
        api_name: "chat",
        version: "v1",
        description: "Manage Chat spaces and messages",
    },
    ServiceEntry {
        aliases: &["vault"],
        api_name: "vault",
        version: "v1",
        description: "Manage eDiscovery holds and exports",
    },
    ServiceEntry {
        aliases: &["groupssettings"],
        api_name: "groupssettings",
        version: "v1",
        description: "Manage Google Groups settings",
    },
    ServiceEntry {
        aliases: &["reseller"],
        api_name: "reseller",
        version: "v1",
        description: "Manage Workspace subscriptions",
    },
    ServiceEntry {
        aliases: &["licensing"],
        api_name: "licensing",
        version: "v1",
        description: "Manage product licenses",
    },
    ServiceEntry {
        aliases: &["apps-script", "script"],
        api_name: "script",
        version: "v1",
        description: "Manage and execute Apps Script projects",
    },
    ServiceEntry {
        aliases: &["classroom"],
        api_name: "classroom",
        version: "v1",
        description: "Manage classes, rosters, and coursework",
    },
    ServiceEntry {
        aliases: &["cloudidentity"],
        api_name: "cloudidentity",
        version: "v1",
        description: "Manage identity groups and memberships",
    },
    ServiceEntry {
        aliases: &["alertcenter"],
        api_name: "alertcenter",
        version: "v1beta1",
        description: "Manage Workspace security alerts",
    },
    ServiceEntry {
        aliases: &["forms"],
        api_name: "forms",
        version: "v1",
        description: "Read and write Google Forms",
    },
    ServiceEntry {
        aliases: &["keep"],
        api_name: "keep",
        version: "v1",
        description: "Manage Google Keep notes",
    },
    ServiceEntry {
        aliases: &["meet"],
        api_name: "meet",
        version: "v2",
        description: "Manage Google Meet conferences",
    },
    ServiceEntry {
        aliases: &["events"],
        api_name: "workspaceevents",
        version: "v1",
        description: "Subscribe to Google Workspace events",
    },
    ServiceEntry {
        aliases: &["modelarmor"],
        api_name: "modelarmor",
        version: "v1",
        description: "Filter user-generated content for safety",
    },
    ServiceEntry {
        aliases: &["workflow", "wf"],
        api_name: "workflow",
        version: "v1",
        description: "Cross-service productivity workflows",
    },
];

/// Resolves a service alias to (api_name, version).
pub fn resolve_service(name: &str) -> Result<(String, String), GwsError> {
    for entry in SERVICES {
        if entry.aliases.contains(&name) {
            return Ok((entry.api_name.to_string(), entry.version.to_string()));
        }
    }
    let all_names: Vec<&str> = SERVICES
        .iter()
        .flat_map(|e| e.aliases.iter().copied())
        .collect();
    Err(GwsError::Validation(format!(
        "Unknown service '{}'. Known services: {}. Use '<api>:<version>' syntax for unlisted APIs.",
        name,
        all_names.join(", ")
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_service_known() {
        assert_eq!(
            resolve_service("drive").unwrap(),
            ("drive".to_string(), "v3".to_string())
        );
        assert_eq!(
            resolve_service("admin").unwrap(),
            ("admin".to_string(), "directory_v1".to_string())
        );
        assert_eq!(
            resolve_service("directory").unwrap(),
            ("admin".to_string(), "directory_v1".to_string())
        );
    }

    #[test]
    fn test_resolve_service_all_aliases() {
        // Verify every alias in SERVICES resolves correctly
        for entry in SERVICES {
            for alias in entry.aliases {
                let (api_name, version) = resolve_service(alias)
                    .unwrap_or_else(|_| panic!("alias '{}' should resolve", alias));
                assert_eq!(api_name, entry.api_name, "alias '{}' wrong api_name", alias);
                assert_eq!(version, entry.version, "alias '{}' wrong version", alias);
            }
        }
    }

    #[test]
    fn test_resolve_service_unknown() {
        let err = resolve_service("unknown_service");
        assert!(err.is_err());
        match err.unwrap_err() {
            GwsError::Validation(msg) => assert!(msg.contains("Unknown service")),
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_resolve_service_case_sensitive() {
        // Service names should be case-sensitive
        assert!(resolve_service("Drive").is_err());
        assert!(resolve_service("GMAIL").is_err());
    }

    #[test]
    fn test_resolve_service_empty_string() {
        assert!(resolve_service("").is_err());
    }

    #[test]
    fn test_resolve_service_error_lists_known_services() {
        let err = resolve_service("nonexistent");
        match err.unwrap_err() {
            GwsError::Validation(msg) => {
                assert!(msg.contains("drive"), "error should list 'drive': {msg}");
                assert!(msg.contains("gmail"), "error should list 'gmail': {msg}");
            }
            _ => panic!("Expected Validation error"),
        }
    }

    #[test]
    fn test_no_duplicate_aliases() {
        let mut seen = std::collections::HashSet::new();
        for entry in SERVICES {
            for alias in entry.aliases {
                assert!(
                    seen.insert(*alias),
                    "duplicate alias '{}' found in SERVICES",
                    alias
                );
            }
        }
    }
}
