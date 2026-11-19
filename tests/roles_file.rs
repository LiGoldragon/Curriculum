//! The committed producer role declaration uses the typed 0.2 identity form.
#![cfg(feature = "datom")]

use curriculum::{generated::RolesDocument, roles};
use datom_codec::{Actualizing, Budget, Potential};
use protos::ReaderBudget;

#[test]
fn committed_roles_file_parses_and_validates_as_typed_roles() {
    let text = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/roles.datom"))
        .expect("committed roles.datom");
    let roles = Potential::<RolesDocument>::from(text)
        .actualize(&mut Budget {
            remaining: 1_048_576,
            reader: ReaderBudget::try_from(1_048_576).unwrap(),
            depth: 0,
            maximum_depth: 1024,
        })
        .expect("typed roles archive");
    let RolesDocument::Roles(roles) = roles;
    assert!(!roles.role_definition_vector.is_empty());
    roles::validate(&roles).expect("committed roles policy");
}
