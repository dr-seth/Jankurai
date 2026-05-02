use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn cell_registry_and_manifest_schemas_parse() {
    let repo = repo_root();
    let manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/cell-manifest.schema.json")).unwrap(),
    )
    .unwrap();
    let registry: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/cell-registry.schema.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(
        manifest["$id"],
        "https://humanlint.dev/schemas/cell-manifest.schema.json"
    );
    assert_eq!(
        registry["$id"],
        "https://humanlint.dev/schemas/cell-registry.schema.json"
    );
    assert_eq!(
        registry["properties"]["cells"]["items"]["$ref"],
        "cell-manifest.schema.json"
    );

    let required = manifest["required"].as_array().unwrap();
    for key in ["cell_id", "version", "lifecycle", "certification_status"] {
        assert!(required.iter().any(|value| value == key));
    }
    assert_eq!(
        manifest["properties"]["proof_lanes"]["items"]["type"],
        "string"
    );

    let proof_receipt: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/proof-receipt.schema.json")).unwrap(),
    )
    .unwrap();
    let proof_plan: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/proof-plan.schema.json")).unwrap(),
    )
    .unwrap();

    assert_eq!(
        proof_receipt["$id"],
        "https://humanlint.dev/schemas/proof-receipt.schema.json"
    );
    assert_eq!(
        proof_plan["$id"],
        "https://humanlint.dev/schemas/proof-plan.schema.json"
    );
    assert_eq!(proof_receipt["properties"]["lane"]["type"], "string");
    assert_eq!(proof_plan["properties"]["changed_paths"]["type"], "array");

    let evidence_index: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/evidence-index.schema.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        evidence_index["$id"],
        "https://humanlint.dev/schemas/evidence-index.schema.json"
    );
    assert_eq!(evidence_index["properties"]["plan_path"]["type"], "string");
    let ei_props = evidence_index["properties"].as_object().unwrap();
    assert!(ei_props.contains_key("ux_qa_report_path"));
    assert!(ei_props.contains_key("security_evidence_path"));
    assert!(ei_props.contains_key("repo_score_json_path"));
    assert!(ei_props.contains_key("sarif_path"));
    assert!(ei_props.contains_key("github_step_summary_path"));
    assert!(ei_props.contains_key("repair_queue_jsonl_path"));
    assert!(ei_props.contains_key("boundaries_manifest_path"));
    let ei_required = evidence_index["required"].as_array().unwrap();
    for key in [
        "ux_qa_report_path",
        "security_evidence_path",
        "repo_score_json_path",
        "sarif_path",
        "github_step_summary_path",
        "repair_queue_jsonl_path",
        "boundaries_manifest_path",
    ] {
        assert!(
            !ei_required.iter().any(|v| v == key),
            "companion path {key} must stay optional"
        );
    }

    let init_profile: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/init-profile.schema.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        init_profile["$id"],
        "https://humanlint.dev/schemas/init-profile.schema.json"
    );

    let security_evidence: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/security-evidence.schema.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        security_evidence["$id"],
        "https://humanlint.dev/schemas/security-evidence.schema.json"
    );
    assert_eq!(security_evidence["properties"]["lane"]["const"], "security");

    let context_pack: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/context-pack.schema.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        context_pack["$id"],
        "https://humanlint.dev/schemas/context-pack.schema.json"
    );

    let repair_plan: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/repair-plan.schema.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        repair_plan["$id"],
        "https://humanlint.dev/schemas/repair-plan.schema.json"
    );
    let rp_required = repair_plan["required"].as_array().unwrap();
    assert!(rp_required.iter().any(|value| value == "packets"));

    let boundaries: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/boundaries.schema.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        boundaries["$id"],
        "https://humanlint.dev/schemas/boundaries.schema.json"
    );
    let b_required = boundaries["required"].as_array().unwrap();
    assert!(b_required.iter().any(|value| value == "stack"));
    assert!(b_required.iter().any(|value| value == "queues"));

    let ux_policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/ux-qa-policy.schema.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        ux_policy["$id"],
        "https://humanlint.dev/schemas/ux-qa-policy.schema.json"
    );
    assert!(ux_policy["properties"].get("artifactRoot").is_some());
    assert!(ux_policy["properties"].get("outputRoot").is_some());

    let ux_report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(repo.join("schemas/ux-qa.schema.json")).unwrap())
            .unwrap();
    assert_eq!(
        ux_report["$id"],
        "https://humanlint.dev/schemas/ux-qa.schema.json"
    );
    let ur_required = ux_report["required"].as_array().unwrap();
    assert!(ur_required.iter().any(|value| value == "reports"));
    assert_eq!(
        ux_report["$defs"]["uxQaReport"]["properties"]["schemaVersion"]["const"],
        "1.2.0"
    );

    let repo_score: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(repo.join("schemas/repo-score.schema.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        repo_score["$id"],
        "https://humanlint.dev/schemas/repo-score.schema.json"
    );
    assert!(repo_score["properties"].get("ux_qa").is_some());
    assert_eq!(
        repo_score["properties"]["ux_qa"]["$ref"],
        "#/$defs/uxQaReadiness"
    );
    let ux_ready = &repo_score["$defs"]["uxQaReadiness"];
    assert!(ux_ready["properties"].get("artifact").is_some());
    assert!(repo_score["properties"].get("security_evidence").is_some());
    assert_eq!(
        repo_score["properties"]["security_evidence"]["$ref"],
        "#/$defs/securityEvidenceReadiness"
    );
    let sec_ready = &repo_score["$defs"]["securityEvidenceReadiness"];
    assert!(sec_ready["properties"].get("artifact").is_some());
    assert!(repo_score["properties"].get("boundaries").is_some());
    assert_eq!(
        repo_score["properties"]["boundaries"]["$ref"],
        "#/$defs/boundariesReadiness"
    );
    let b_ready = &repo_score["$defs"]["boundariesReadiness"];
    assert!(b_ready["properties"].get("artifact").is_some());
}
