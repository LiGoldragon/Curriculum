//! The 0.2.0 newtypes keep the wire and the text of the type aliases they replace.
//!
//! `support/previous_478c.rs` is `src/generated.rs` at 478cbc5 byte for byte, so the old shapes,
//! field order and variant order, `Outcome::Rejected(String)` at index four among them, are the
//! reviewed ones and not a surrogate. Each sample is built once in the old family and once in the
//! new, from the same data, and the archived bytes (and, with `datom`, the text) must be equal.
//! The samples are representative values, not a proof for every value.

#[path = "support/previous_478c.rs"]
#[allow(clippy::all, dead_code)]
mod previous;

use curriculum::generated as current;
use rkyv::rancor::Error;

macro_rules! samples {
    ($m:ident, $wrap:ident) => {{
        use $m::*;
        let identity = |name: &str| $wrap!($m, ModuleIdentity, name.to_string());
        let provenance = || ContextProvenance {
            module_identity: identity("spirit:core"),
            repository_name: $wrap!($m, RepositoryName, "psyche-belief".to_string()),
            source_revision: $wrap!($m, SourceRevision, "a".repeat(40)),
            source_relative_path: $wrap!($m, SourceRelativePath, "spirit/core.md".to_string()),
            source_sha256: $wrap!($m, SourceSha256, "1".repeat(64)),
            body_sha256: $wrap!($m, BodySha256, "2".repeat(64)),
            rendered_sha256: $wrap!($m, RenderedSha256, "3".repeat(64)),
            startup_placement: StartupPlacement::System,
            integer: 7,
        };
        let caller = || CallerMetaflow {
            first_string: "Field".into(),
            second_string: "Publisher".into(),
            third_string: "Quaternary".into(),
        };
        let selection = StartupSelection {
            caller_metaflow: caller(),
            target: Target::Claude,
            system_modules: $wrap!($m, SystemModules, vec![identity("spirit:core"), identity("intent:seat")]),
            user_modules: $wrap!($m, UserModules, vec![identity("vision:core")]),
            flow_id_vector: vec![$wrap!($m, FlowId, "first".to_string())],
        };
        let role_selection = RoleSelection {
            role_name: $wrap!($m, RoleName, "illustrator".to_string()),
            surface: Surface::ClaudeAgent,
        };
        let resolved = ResolvedModule {
            module_identity: identity("vision:book"),
            repository_name: $wrap!($m, RepositoryName, "mind-belief".to_string()),
            source_revision: $wrap!($m, SourceRevision, "b".repeat(40)),
            source_root: $wrap!($m, SourceRoot, "/srv/mind".to_string()),
            source_relative_path: $wrap!($m, SourceRelativePath, "vision/book.md".to_string()),
            module_type: $wrap!($m, ModuleType, "vision".to_string()),
            module_subject: $wrap!($m, ModuleSubject, "book".to_string()),
            declared_placement: $wrap!($m, DeclaredPlacement, "system".to_string()),
            source_sha256: $wrap!($m, SourceSha256, "4".repeat(64)),
            body_sha256: $wrap!($m, BodySha256, "5".repeat(64)),
            string: "Exact text.\n".into(),
            string_vector: vec!["dependency".into()],
        };
        let startup = StartupComposition {
            system_text: $wrap!($m, SystemText, "system\n".to_string()),
            user_text: $wrap!($m, UserText, "user\n".to_string()),
            records_text: $wrap!($m, RecordsText, "records\n".to_string()),
            system_payload_sha256: $wrap!($m, SystemPayloadSha256, "6".repeat(64)),
            user_payload_sha256: $wrap!($m, UserPayloadSha256, "7".repeat(64)),
            records_payload_sha256: $wrap!($m, RecordsPayloadSha256, "8".repeat(64)),
            context_provenance_vector: vec![provenance()],
        };
        let role = RoleComposition {
            role_name: $wrap!($m, RoleName, "illustrator".to_string()),
            surface: Surface::ClaudeAgent,
            model_id: $wrap!($m, ModelId, "sonnet".to_string()),
            effort: Effort::Medium,
            permission: Permission::Unrestricted,
            description_text: $wrap!($m, DescriptionText, "description".to_string()),
            context_text: $wrap!($m, ContextText, "context".to_string()),
            role_tools: RoleTools::Explicit(vec!["Read".into(), "Bash".into()]),
            role_callers: RoleCallers::Only(vec![caller()]),
            context_provenance_vector: vec![provenance()],
            description_payload_sha256: $wrap!($m, DescriptionPayloadSha256, "9".repeat(64)),
            context_payload_sha256: $wrap!($m, ContextPayloadSha256, "0".repeat(64)),
        };
        let definition = RoleDefinition {
            role_name: $wrap!($m, RoleName, "illustrator".to_string()),
            surface: Surface::ClaudeAgent,
            model_id: $wrap!($m, ModelId, "sonnet".to_string()),
            effort: Effort::Medium,
            permission: Permission::Unrestricted,
            description_module: $wrap!($m, DescriptionModule, identity("role:illustrator-description")),
            context_modules: $wrap!($m, ContextModules, vec![identity("role:illustrator-svg"), identity("compensation:illustration")]),
            role_tools: RoleTools::Inherit,
            role_callers: RoleCallers::Any,
        };
        let projection = ProjectionCandidate {
            deployment_destination: $wrap!($m, DeploymentDestination, "/workspace".to_string()),
            repository_revision_vector: vec![RepositoryRevision {
                repository_name: $wrap!($m, RepositoryName, "psyche-context".to_string()),
                source_revision: $wrap!($m, SourceRevision, "c".repeat(40)),
            }],
            published_inventory_digest: $wrap!($m, PublishedInventoryDigest, "d".repeat(64)),
            published_content_digest: $wrap!($m, PublishedContentDigest, "e".repeat(64)),
            renderer_version: $wrap!($m, RendererVersion, "curriculum-workspace-projector-v1".to_string()),
        };
        Samples {
            queries: vec![
                Query::ResolveSkills(vec![identity("spirit:core")]),
                Query::ResolveSources(vec![identity("vision:book"), identity("spirit:core")]),
                Query::ComposeStartup(selection.clone()),
                Query::ComposeRole(role_selection.clone()),
            ],
            requests: vec![
                CliRequest::ResolveSkills(vec![identity("spirit:core")]),
                CliRequest::ResolveSources(vec![identity("vision:book")]),
                CliRequest::ComposeStartup(selection),
                CliRequest::ComposeRole(role_selection),
            ],
            outcomes: vec![
                Outcome::ResolvedSkills(vec![identity("spirit:core")]),
                Outcome::ResolvedSources(vec![resolved]),
                Outcome::StartupComposed(startup),
                Outcome::RoleComposed(role),
                Outcome::Rejected("no such module".into()),
            ],
            documents: vec![RolesDocument::Roles(Roles { role_definition_vector: vec![definition] })],
            projections: vec![projection],
        }
    }};
}

macro_rules! old_wrap {
    ($m:ident, $t:ident, $e:expr) => {
        $e
    };
}
macro_rules! new_wrap {
    ($m:ident, $t:ident, $e:expr) => {
        $m::$t($e)
    };
}

struct Samples<Q, R, O, D, P> {
    queries: Vec<Q>,
    requests: Vec<R>,
    outcomes: Vec<O>,
    documents: Vec<D>,
    projections: Vec<P>,
}

fn old_samples() -> Samples<previous::Query, previous::CliRequest, previous::Outcome, previous::RolesDocument, previous::ProjectionCandidate> {
    samples!(previous, old_wrap)
}

fn new_samples() -> Samples<current::Query, current::CliRequest, current::Outcome, current::RolesDocument, current::ProjectionCandidate> {
    samples!(current, new_wrap)
}

macro_rules! archived {
    ($value:expr) => {
        rkyv::to_bytes::<Error>($value).expect("archive").to_vec()
    };
}

macro_rules! assert_equal_bytes {
    ($label:expr, $old:expr, $new:expr) => {{
        assert_eq!($old.len(), $new.len(), "{}", $label);
        for (index, (old, new)) in $old.iter().zip($new.iter()).enumerate() {
            assert_eq!(archived!(old), archived!(new), "{} sample {}", $label, index);
        }
    }};
}

#[test]
fn the_current_types_archive_to_the_bytes_of_the_478c_types() {
    let (old, new) = (old_samples(), new_samples());
    assert_equal_bytes!("Query", old.queries, new.queries);
    assert_equal_bytes!("CliRequest", old.requests, new.requests);
    assert_equal_bytes!("Outcome", old.outcomes, new.outcomes);
    assert_equal_bytes!("RolesDocument", old.documents, new.documents);
    assert_equal_bytes!("ProjectionCandidate", old.projections, new.projections);
}

#[test]
fn an_old_archive_restores_as_the_current_value_and_the_variant_order_is_kept() {
    let (old, new) = (old_samples(), new_samples());
    for (index, (old, new)) in old.outcomes.iter().zip(&new.outcomes).enumerate() {
        let restored = rkyv::from_bytes::<current::Outcome, Error>(&archived!(old)).expect("restore as current");
        assert_eq!(&restored, new, "Outcome sample {index}");
    }
    let rejected = rkyv::from_bytes::<current::Outcome, Error>(&archived!(&previous::Outcome::Rejected("x".into()))).unwrap();
    assert_eq!(rejected, current::Outcome::Rejected("x".into()), "Rejected stays the fifth variant");
    for (index, (old, new)) in old.queries.iter().zip(&new.queries).enumerate() {
        let restored = rkyv::from_bytes::<current::Query, Error>(&archived!(old)).expect("restore as current");
        assert_eq!(&restored, new, "Query sample {index}");
    }
}

#[cfg(feature = "datom")]
mod text {
    use super::*;
    use datom_codec::{Datomizable, Composing, Actualizing, Budget, Potential};
    use protos::{Protosizable, ReaderBudget, Textualizable};

    fn written<T: Datomizable>(value: &T) -> String {
        value.datomize(Vec::new()).protosize().textualize()
    }

    fn read<T: Composing>(text: &str) -> T {
        let mut budget = Budget {
            remaining: 100_000,
            reader: ReaderBudget::try_from(100_000usize).unwrap(),
            depth: 0,
            maximum_depth: 128,
        };
        Potential::<T>::from(text).actualize(&mut budget).expect("read the datom text")
    }

    #[test]
    fn the_current_types_write_the_text_of_the_478c_types_and_read_it_back() {
        let (old, new) = (old_samples(), new_samples());
        for (index, (old, new)) in old.outcomes.iter().zip(&new.outcomes).enumerate() {
            let text = written(old);
            assert_eq!(text, written(new), "Outcome sample {index}");
            assert_eq!(&read::<current::Outcome>(&text), new, "Outcome sample {index}");
        }
        for (index, (old, new)) in old.requests.iter().zip(&new.requests).enumerate() {
            let text = written(old);
            assert_eq!(text, written(new), "CliRequest sample {index}");
            assert_eq!(&read::<current::CliRequest>(&text), new, "CliRequest sample {index}");
        }
    }
}
