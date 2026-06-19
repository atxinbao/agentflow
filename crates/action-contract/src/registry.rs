use std::collections::BTreeMap;

use agentflow_ontology::OntologyRegistry;

use crate::model::{ActionContract, ActionContractBundle, ActionTypeDefinition};
use crate::report::ActionContractValidationReport;
use crate::validation::validate_action_contract_bundle;

#[derive(Debug, Clone)]
pub struct ActionContractRegistry {
    bundle: ActionContractBundle,
    action_types: BTreeMap<String, ActionTypeDefinition>,
    contracts: BTreeMap<String, ActionContract>,
}

impl ActionContractRegistry {
    pub fn load_bundle(
        bundle: ActionContractBundle,
        ontology_registry: &OntologyRegistry,
    ) -> Result<Self, ActionContractValidationReport> {
        let report = validate_action_contract_bundle(&bundle, ontology_registry);
        if !report.valid {
            return Err(report);
        }
        let action_types = bundle
            .action_types
            .iter()
            .cloned()
            .map(|item| (item.id.clone(), item))
            .collect();
        let contracts = bundle
            .contracts
            .iter()
            .cloned()
            .map(|item| (item.action_type.clone(), item))
            .collect();

        Ok(Self {
            bundle,
            action_types,
            contracts,
        })
    }

    pub fn bundle(&self) -> &ActionContractBundle {
        &self.bundle
    }

    pub fn list_action_types(&self) -> Vec<&ActionTypeDefinition> {
        self.action_types.values().collect()
    }

    pub fn get_action_type(&self, action_type: &str) -> Option<&ActionTypeDefinition> {
        self.action_types.get(action_type)
    }

    pub fn get_action_contract(&self, action_type: &str, version: &str) -> Option<&ActionContract> {
        if self.bundle.definition_version != version {
            return None;
        }
        self.contracts.get(action_type)
    }
}
