use curriculum::generated::{Effort, Permission, RolePacketPlan, Surface};
use curriculum::roles::RendersRolePacketPlan;

fn packet(name: &str, permission: Permission) -> RolePacketPlan {
    RolePacketPlan {
        role_policy_option: None,
        surface: Surface::ClaudeAgent,
        first_string: format!(".claude/agents/{name}.md"),
        second_string: name.into(),
        third_string: "Fixture role".into(),
        fourth_string: "sonnet".into(),
        effort: Effort::Medium,
        permission,
        fifth_string: "UNIVERSAL_CONTEXT".into(),
        sixth_string: "EXACT_AUTHORED_PROCEDURE".into(),
    }
}

#[test]
fn renderer_has_only_its_procedure_and_publishing_tools() {
    let rendered = packet("book", Permission::Unrestricted).render(&["STANDING_CONTEXT".into()]);
    assert!(rendered.contains("tools: Read, Bash, Agent, Artifact\n"));
    assert!(rendered.ends_with("EXACT_AUTHORED_PROCEDURE"));
    assert!(!rendered.contains("UNIVERSAL_CONTEXT"));
    assert!(!rendered.contains("STANDING_CONTEXT"));
    assert!(!rendered.contains("ToolSearch"));
}

#[test]
fn page_updater_preserves_its_context_and_data_tools() {
    let rendered =
        packet("book-page", Permission::Unrestricted).render(&["STANDING_CONTEXT".into()]);
    assert!(rendered.contains("ArtifactData, ArtifactComments"));
    assert!(rendered.contains("UNIVERSAL_CONTEXT"));
    assert!(rendered.contains("STANDING_CONTEXT"));
    assert!(rendered.ends_with("EXACT_AUTHORED_PROCEDURE"));
}

#[test]
fn ordinary_reader_has_an_explicit_nonpublishing_tool_set() {
    let rendered =
        packet("read-ordinary", Permission::Restricted).render(&["STANDING_CONTEXT".into()]);
    assert!(rendered.contains("tools: Read, Grep, Glob, Bash, WebFetch, WebSearch, LSP\n"));
    assert!(!rendered.contains("Artifact"));
    assert!(rendered.contains("UNIVERSAL_CONTEXT"));
    assert!(rendered.contains("STANDING_CONTEXT"));
}
