use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use crate::model::{
    LinkTypeDefinition, ObjectTypeDefinition, OntologyBundle, OntologyDefinitionRecord,
    OntologyValidationReport,
};
use crate::validation::validate_ontology_bundle;

#[derive(Debug, Clone)]
pub struct OntologyRegistry {
    bundle: OntologyBundle,
    object_types: BTreeMap<String, ObjectTypeDefinition>,
    link_types: BTreeMap<String, LinkTypeDefinition>,
    definition_records: BTreeMap<String, OntologyDefinitionRecord>,
}

impl OntologyRegistry {
    pub fn load_bundle(bundle: OntologyBundle) -> Result<Self, OntologyValidationReport> {
        let report = validate_ontology_bundle(&bundle);
        if !report.valid {
            return Err(report);
        }

        let object_types = bundle
            .object_types
            .iter()
            .cloned()
            .map(|definition| (definition.id.clone(), definition))
            .collect();
        let link_types = bundle
            .link_types
            .iter()
            .cloned()
            .map(|definition| (definition.id.clone(), definition))
            .collect();
        let definition_records = bundle
            .definition_records
            .iter()
            .cloned()
            .map(|record| (record.id.clone(), record))
            .collect();

        Ok(Self {
            bundle,
            object_types,
            link_types,
            definition_records,
        })
    }

    pub fn bundle(&self) -> &OntologyBundle {
        &self.bundle
    }

    pub fn validate_bundle(&self) -> OntologyValidationReport {
        validate_ontology_bundle(&self.bundle)
    }

    pub fn ontology_ref(&self) -> String {
        format!(
            "{}@{}",
            self.bundle.namespace, self.bundle.definition_version
        )
    }

    pub fn list_object_types(&self) -> Vec<&ObjectTypeDefinition> {
        self.object_types.values().collect()
    }

    pub fn list_link_types(&self) -> Vec<&LinkTypeDefinition> {
        self.link_types.values().collect()
    }

    pub fn get_object_type(&self, object_type_id: &str) -> Option<&ObjectTypeDefinition> {
        self.object_types.get(object_type_id)
    }

    pub fn get_link_type(&self, link_type_id: &str) -> Option<&LinkTypeDefinition> {
        self.link_types.get(link_type_id)
    }

    pub fn get_definition_record(&self, record_id: &str) -> Option<&OntologyDefinitionRecord> {
        self.definition_records.get(record_id)
    }

    pub fn validate_object_ref(&self, object_type_id: &str) -> Result<()> {
        if self.object_types.contains_key(object_type_id) {
            Ok(())
        } else {
            Err(anyhow!("object type `{object_type_id}` is not defined"))
        }
    }

    pub fn validate_link_endpoint(
        &self,
        link_type_id: &str,
        source_type: &str,
        target_type: &str,
    ) -> Result<()> {
        let link = self
            .get_link_type(link_type_id)
            .ok_or_else(|| anyhow!("link type `{link_type_id}` is not defined"))?;
        if link.source_object_type != source_type {
            return Err(anyhow!(
                "link `{link_type_id}` requires source `{}` but got `{source_type}`",
                link.source_object_type
            ));
        }
        if link.target_object_type != target_type {
            return Err(anyhow!(
                "link `{link_type_id}` requires target `{}` but got `{target_type}`",
                link.target_object_type
            ));
        }
        Ok(())
    }
}
