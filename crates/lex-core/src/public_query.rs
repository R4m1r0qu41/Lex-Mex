//! Public transport contract for the fixture-first retrieval interface.
//! These DTOs are not raw DB records and do not authorize a public export.
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub id: String,
    pub corpus_commit: String,
    pub fixture: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Instrument {
    pub slug: String,
    pub id: String,
    pub title: String,
    pub short_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub official_url: String,
    pub publisher: String,
    pub retrieved_at: String,
    pub source_sha256: String,
    pub recorded_reform_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Article {
    pub id: String,
    pub instrument: Instrument,
    pub provision_type: String,
    pub number: String,
    pub label: String,
    pub text: String,
    pub text_sha256: String,
    pub source: Source,
    pub temporal_status: String,
    pub review_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchHit {
    pub id: String,
    pub instrument: Instrument,
    pub provision_type: String,
    pub number: String,
    pub label: String,
    pub excerpt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub slug: String,
    pub provision_type: String,
    pub number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub target_id: String,
    pub target: Option<Target>,
    pub source_span: String,
    pub resolution_status: String,
    pub resolution_rule: Option<String>,
    pub qualifiers: Vec<Qualifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Qualifier {
    #[serde(rename = "type")]
    pub qualifier_type: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "data",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum QueryResult {
    Catalog {
        items: Vec<Instrument>,
        next_cursor: Option<String>,
    },
    Search {
        query: String,
        items: Vec<SearchHit>,
        next_cursor: Option<String>,
    },
    Article {
        provision: Box<Article>,
    },
    References {
        items: Vec<Edge>,
        next_cursor: Option<String>,
    },
    Coverage {
        scope: String,
        notice: String,
    },
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicResponse {
    pub schema_version: String,
    pub release: Release,
    pub result: QueryResult,
}

/// The compiled schema is built once; a build failure is cached and reported
/// on every call, so rejection behavior is unchanged.
fn validator() -> Result<&'static jsonschema::Validator, String> {
    static VALIDATOR: std::sync::OnceLock<Result<jsonschema::Validator, String>> =
        std::sync::OnceLock::new();
    VALIDATOR
        .get_or_init(|| {
            let schema: serde_json::Value =
                serde_json::from_str(include_str!("../../../schemas/public-query-v1.schema.json"))
                    .map_err(|e| e.to_string())?;
            jsonschema::validator_for(&schema).map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)
}

/// Validate the closed public response shape and exact UTF-8 article digest.
/// Production projection/parity checks remain the publication owner's task.
pub fn validate_response(bytes: &[u8]) -> Result<PublicResponse, String> {
    let response: PublicResponse = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    let value: serde_json::Value = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
    validator()?.validate(&value).map_err(|e| e.to_string())?;
    if let QueryResult::Article { provision } = &response.result
        && format!("{:x}", Sha256::digest(provision.text.as_bytes())) != provision.text_sha256
    {
        return Err("Public article text digest mismatch".into());
    }
    if let QueryResult::Article { provision } = &response.result {
        let source = url::Url::parse(&provision.source.official_url).map_err(|e| e.to_string())?;
        // Recorded official URLs are preserved as published: http or https,
        // never credentials or another scheme (the JS contract and the schema
        // apply the same rule).
        if !matches!(source.scheme(), "http" | "https")
            || !source.username().is_empty()
            || source.password().is_some()
        {
            return Err("Invalid public source URL".into());
        }
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture_response() -> serde_json::Value {
        let f: serde_json::Value = serde_json::from_str(include_str!(
            "../../../reader/fixtures/public-query-v1.json"
        ))
        .unwrap();
        serde_json::json!({"schema_version":"public-retrieval-v1","release":f["release"],
            "result":{"kind":"article","data":{"provision":f["provisions"][1]}}})
    }
    #[test]
    fn synthetic_unicode_fixture_matches_closed_contract_and_digest() {
        let response = fixture_response();
        let typed = validate_response(&serde_json::to_vec(&response).unwrap()).unwrap();
        validate_response(&serde_json::to_vec(&typed).unwrap()).unwrap();
    }
    #[test]
    fn actual_node_transport_fixtures_conform_to_rust_and_schema() {
        let cases: serde_json::Value = serde_json::from_str(include_str!(
            "../../../reader/fixtures/public-query-responses-v1.json"
        ))
        .unwrap();
        for case in cases.as_array().unwrap() {
            validate_response(&serde_json::to_vec(&case["body"]).unwrap()).unwrap();
        }
    }
    #[test]
    fn source_urls_follow_the_shared_policy() {
        let with_url = |url: &str| {
            let mut response = fixture_response();
            response["result"]["data"]["provision"]["source"]["official_url"] =
                serde_json::json!(url);
            validate_response(&serde_json::to_vec(&response).unwrap())
        };
        assert!(with_url("http://www.oag.salud.gob.mx/descargas/LV/56-31052021.pdf").is_ok());
        assert!(with_url("https://www.diputados.gob.mx/LeyesBiblio/pdf/LBOGM.pdf").is_ok());
        for bad in [
            "https://user:pw@example.org/a.pdf",
            "ftp://example.org/a.pdf",
            "javascript:alert(1)",
            "//example.org/a.pdf",
        ] {
            assert!(with_url(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn a_resolved_reference_may_have_no_public_route_target() {
        // Review finding R-002: a canonical target whose number has no public route
        // keeps its exact id and `resolved` status with a null target.
        let mut response = fixture_response();
        response["result"] = serde_json::json!({"kind":"references","data":{"items":[
            {"target_id":"urn:lex-mx:federal:statute:lft:transitory:9º","target":null,
             "source_span":"transitorio 9º","resolution_status":"resolved","resolution_rule":null,"qualifiers":[]},
            {"target_id":"urn:lex-mx:federal:statute:lft:article:3","target":{"slug":"lft","provision_type":"article","number":"3"},
             "source_span":"artículo 3","resolution_status":"resolved","resolution_rule":null,"qualifiers":[]}
        ],"next_cursor":null}});
        validate_response(&serde_json::to_vec(&response).unwrap()).unwrap();
    }
    #[test]
    fn rejects_private_fields_tampering_and_live_historical_target() {
        let mut response = fixture_response();
        response["result"]["data"]["provision"]["operator_note"] = serde_json::json!("private");
        assert!(validate_response(&serde_json::to_vec(&response).unwrap()).is_err());
        response = fixture_response();
        response["result"]["data"]["provision"]["text"] = serde_json::json!("changed");
        assert!(validate_response(&serde_json::to_vec(&response).unwrap()).is_err());
        response["result"] = serde_json::json!({"kind":"references","data":{"items":[{
            "target_id":"historical","target":{"slug":"test","provision_type":"article","number":"1"},
            "source_span":"test","resolution_status":"historical_target_unavailable","resolution_rule":null,"qualifiers":[]
        }],"next_cursor":null}});
        assert!(validate_response(&serde_json::to_vec(&response).unwrap()).is_err());
    }
}
