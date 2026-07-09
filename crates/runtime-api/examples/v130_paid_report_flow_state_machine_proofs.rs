use agentflow_runtime_api::{
    paid_report_flow_state_machine, PAID_REPORT_FLOW_STATE_MACHINE_VERSION,
};
use anyhow::{bail, Context, Result};
use serde::Serialize;
use std::{collections::HashSet, fs, path::Path};

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.len() != 1 {
        bail!("usage: v130_paid_report_flow_state_machine_proofs <state-machine>");
    }

    let machine = paid_report_flow_state_machine();
    let mut failures = Vec::new();

    if machine.version != PAID_REPORT_FLOW_STATE_MACHINE_VERSION {
        failures.push("wrong-state-machine-version".to_string());
    }
    if machine.status != "passed" {
        failures.push("state-machine-status-not-passed".to_string());
    }
    if machine.release_version != "v1.3.0" {
        failures.push("wrong-release-version".to_string());
    }

    let states = machine
        .states
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    for state in [
        "draft-order",
        "order-ready",
        "authorized",
        "admitted",
        "running",
        "artifact-ready",
        "evidence-complete",
        "accepted",
        "delivery-ready",
        "feedback-needed",
        "repair-requested",
        "rerun-needs-authorization",
        "refunded",
        "expired",
        "closed",
    ] {
        if !states.contains(state) {
            failures.push(format!("missing-state-{state}"));
        }
    }

    if machine.positive_fixtures.is_empty() {
        failures.push("missing-positive-fixtures".to_string());
    }
    if machine.negative_fixtures.is_empty() {
        failures.push("missing-negative-fixtures".to_string());
    }

    for fixture in &machine.positive_fixtures {
        let transition = &fixture.transition;
        if fixture.status != "passed" {
            failures.push(format!("{}-not-passed", fixture.fixture_id));
        }
        if !states.contains(transition.from_state.as_str()) {
            failures.push(format!("{}-unknown-from-state", fixture.fixture_id));
        }
        if !states.contains(transition.to_state.as_str()) {
            failures.push(format!("{}-unknown-to-state", fixture.fixture_id));
        }
        if transition.required_contracts.is_empty() {
            failures.push(format!("{}-missing-required-contracts", fixture.fixture_id));
        }
        if transition
            .required_contracts
            .iter()
            .any(|binding| binding.contract_version.trim().is_empty())
        {
            failures.push(format!("{}-missing-contract-version", fixture.fixture_id));
        }
    }

    if !machine.positive_fixtures.iter().any(|fixture| {
        fixture.transition.to_state == "accepted" && fixture.transition.writes_accepted_authority
    }) {
        failures.push("missing-accepted-authority-positive-transition".to_string());
    }
    if !machine.positive_fixtures.iter().any(|fixture| {
        fixture.transition.to_state == "delivery-ready"
            && fixture.transition.writes_delivery_ready_authority
    }) {
        failures.push("missing-delivery-ready-authority-positive-transition".to_string());
    }

    for fixture in &machine.negative_fixtures {
        let transition = &fixture.transition;
        if fixture.status != "failed-as-expected" {
            failures.push(format!("{}-unexpected-negative-status", fixture.fixture_id));
        }
        if transition.failure_reasons.is_empty() {
            failures.push(format!("{}-missing-failure-reasons", fixture.fixture_id));
        }
        if transition.writes_authority {
            failures.push(format!("{}-writes-authority", fixture.fixture_id));
        }
        if transition.writes_accepted_authority {
            failures.push(format!("{}-writes-accepted-authority", fixture.fixture_id));
        }
        if transition.writes_delivery_ready_authority {
            failures.push(format!(
                "{}-writes-delivery-ready-authority",
                fixture.fixture_id
            ));
        }
        if transition
            .failure_reasons
            .iter()
            .any(|reason| !reason.prevents_authority_writes)
        {
            failures.push(format!(
                "{}-failure-does-not-block-authority",
                fixture.fixture_id
            ));
        }
    }

    if !failures.is_empty() {
        bail!("v130 paid report flow state machine failed: {failures:?}");
    }

    write_json(Path::new(&args[0]), &machine)
}

fn write_json(path: &Path, payload: &impl Serialize) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create proof directory {}", parent.display()))?;
    }
    fs::write(
        path,
        serde_json::to_string_pretty(payload).context("serialize proof payload")? + "\n",
    )
    .with_context(|| format!("write proof {}", path.display()))?;
    Ok(())
}
