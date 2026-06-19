use agentflow_action_contract::ActionRef;
use serde::{Deserialize, Serialize};

use crate::model::{ArbitrationContext, ObjectLock, ObjectLockKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LockDecision {
    pub available: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_lock: Option<ObjectLock>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl LockDecision {
    pub fn available() -> Self {
        Self {
            available: true,
            blocking_lock: None,
            reason: None,
        }
    }

    pub fn unavailable(blocking_lock: ObjectLock, reason: impl Into<String>) -> Self {
        Self {
            available: false,
            blocking_lock: Some(blocking_lock),
            reason: Some(reason.into()),
        }
    }
}

pub fn default_lock_kind_for_object(object_type: &str) -> ObjectLockKind {
    match object_type {
        "Issue" => ObjectLockKind::RunExecution,
        "Audit" => ObjectLockKind::AuditReview,
        "Spec" => ObjectLockKind::DecisionPending,
        _ => ObjectLockKind::Write,
    }
}

pub fn check_object_lock(
    target: &ActionRef,
    requested_kind: ObjectLockKind,
    context: &ArbitrationContext,
) -> LockDecision {
    let blocking_lock = context.object_locks.iter().find(|lock| {
        lock.object_type == target.object_type
            && lock.object_id == target.id
            && lock_conflicts(lock.lock_kind, requested_kind)
    });

    match blocking_lock {
        Some(lock) => LockDecision::unavailable(
            lock.clone(),
            format!(
                "active {:?} lock already held on {}:{}",
                lock.lock_kind, lock.object_type, lock.object_id
            ),
        ),
        None => LockDecision::available(),
    }
}

fn lock_conflicts(existing: ObjectLockKind, requested: ObjectLockKind) -> bool {
    matches!(
        (existing, requested),
        (ObjectLockKind::Write, _)
            | (_, ObjectLockKind::Write)
            | (ObjectLockKind::RunExecution, _)
            | (_, ObjectLockKind::RunExecution)
            | (ObjectLockKind::AuditReview, ObjectLockKind::AuditReview)
            | (ObjectLockKind::DecisionPending, _)
            | (_, ObjectLockKind::DecisionPending)
    )
}
