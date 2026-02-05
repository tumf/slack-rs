//! Sorting functionality for conversations

use crate::api::ApiResponse;

/// Sort key for conversation list
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Name,
    Created,
    NumMembers,
}

impl SortKey {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "name" => Ok(SortKey::Name),
            "created" => Ok(SortKey::Created),
            "num_members" => Ok(SortKey::NumMembers),
            _ => Err(format!(
                "Invalid sort key '{}'. Valid values: name, created, num_members",
                s
            )),
        }
    }
}

/// Sort direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "asc" => Ok(SortDirection::Asc),
            "desc" => Ok(SortDirection::Desc),
            _ => Err(format!(
                "Invalid sort direction '{}'. Valid values: asc, desc",
                s
            )),
        }
    }
}

/// Sort conversations by the specified key and direction
pub fn sort_conversations(response: &mut ApiResponse, key: SortKey, direction: SortDirection) {
    if let Some(channels) = response.data.get_mut("channels") {
        if let Some(channels_array) = channels.as_array_mut() {
            channels_array.sort_by(|a, b| {
                let ordering = match key {
                    SortKey::Name => {
                        let a_name = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        let b_name = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                        a_name.cmp(b_name)
                    }
                    SortKey::Created => {
                        let a_created = a.get("created").and_then(|v| v.as_i64()).unwrap_or(0);
                        let b_created = b.get("created").and_then(|v| v.as_i64()).unwrap_or(0);
                        a_created.cmp(&b_created)
                    }
                    SortKey::NumMembers => {
                        let a_members = a.get("num_members").and_then(|v| v.as_i64()).unwrap_or(0);
                        let b_members = b.get("num_members").and_then(|v| v.as_i64()).unwrap_or(0);
                        a_members.cmp(&b_members)
                    }
                };

                match direction {
                    SortDirection::Asc => ordering,
                    SortDirection::Desc => ordering.reverse(),
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_sort_key_parse() {
        assert_eq!(SortKey::parse("name").unwrap(), SortKey::Name);
        assert_eq!(SortKey::parse("created").unwrap(), SortKey::Created);
        assert_eq!(SortKey::parse("num_members").unwrap(), SortKey::NumMembers);
        assert!(SortKey::parse("invalid").is_err());
    }

    #[test]
    fn test_sort_direction_parse() {
        assert_eq!(SortDirection::parse("asc").unwrap(), SortDirection::Asc);
        assert_eq!(SortDirection::parse("desc").unwrap(), SortDirection::Desc);
        assert!(SortDirection::parse("invalid").is_err());
    }

    #[test]
    fn test_sort_conversations_by_name() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "zebra"},
                    {"id": "C2", "name": "alpha"},
                    {"id": "C3", "name": "beta"},
                ]),
            )]),
            error: None,
        };

        sort_conversations(&mut response, SortKey::Name, SortDirection::Asc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(channels[0].get("name").unwrap().as_str().unwrap(), "alpha");
        assert_eq!(channels[1].get("name").unwrap().as_str().unwrap(), "beta");
        assert_eq!(channels[2].get("name").unwrap().as_str().unwrap(), "zebra");
    }

    #[test]
    fn test_sort_conversations_by_name_desc() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "alpha"},
                    {"id": "C2", "name": "zebra"},
                    {"id": "C3", "name": "beta"},
                ]),
            )]),
            error: None,
        };

        sort_conversations(&mut response, SortKey::Name, SortDirection::Desc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(channels[0].get("name").unwrap().as_str().unwrap(), "zebra");
        assert_eq!(channels[1].get("name").unwrap().as_str().unwrap(), "beta");
        assert_eq!(channels[2].get("name").unwrap().as_str().unwrap(), "alpha");
    }

    #[test]
    fn test_sort_conversations_by_created() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "newest", "created": 300},
                    {"id": "C2", "name": "oldest", "created": 100},
                    {"id": "C3", "name": "middle", "created": 200},
                ]),
            )]),
            error: None,
        };

        sort_conversations(&mut response, SortKey::Created, SortDirection::Asc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(channels[0].get("created").unwrap().as_i64().unwrap(), 100);
        assert_eq!(channels[1].get("created").unwrap().as_i64().unwrap(), 200);
        assert_eq!(channels[2].get("created").unwrap().as_i64().unwrap(), 300);
    }

    #[test]
    fn test_sort_conversations_by_num_members() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "large", "num_members": 100},
                    {"id": "C2", "name": "small", "num_members": 10},
                    {"id": "C3", "name": "medium", "num_members": 50},
                ]),
            )]),
            error: None,
        };

        sort_conversations(&mut response, SortKey::NumMembers, SortDirection::Asc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        assert_eq!(
            channels[0].get("num_members").unwrap().as_i64().unwrap(),
            10
        );
        assert_eq!(
            channels[1].get("num_members").unwrap().as_i64().unwrap(),
            50
        );
        assert_eq!(
            channels[2].get("num_members").unwrap().as_i64().unwrap(),
            100
        );
    }

    #[test]
    fn test_sort_conversations_missing_fields() {
        let mut response = ApiResponse {
            ok: true,
            data: HashMap::from([(
                "channels".to_string(),
                json!([
                    {"id": "C1", "name": "has_members", "num_members": 50},
                    {"id": "C2", "name": "no_members"},
                    {"id": "C3", "name": "also_has", "num_members": 10},
                ]),
            )]),
            error: None,
        };

        sort_conversations(&mut response, SortKey::NumMembers, SortDirection::Asc);

        let channels = response.data.get("channels").unwrap().as_array().unwrap();
        // Missing field treated as 0, so it should be first
        assert_eq!(
            channels[0].get("name").unwrap().as_str().unwrap(),
            "no_members"
        );
        assert_eq!(
            channels[1].get("num_members").unwrap().as_i64().unwrap(),
            10
        );
        assert_eq!(
            channels[2].get("num_members").unwrap().as_i64().unwrap(),
            50
        );
    }
}
