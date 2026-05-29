use crate::cosmogram::CosmogramState;

#[derive(Debug)]
pub enum AuditError {
    Anomaly(String),
}

pub fn audit_state(state: &CosmogramState) -> Result<(), AuditError> {
    // Red-team check: quarantine if odu_id 0 with machine_only access
    if state.odu_id == 0 && state.access_class == crate::cosmogram::AccessClass::MachineOnly {
        return Err(AuditError::Anomaly("odu 0 cannot be machine_only".into()));
    }
    Ok(())
}
