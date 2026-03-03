#![forbid(unsafe_code)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuredSchemaId {
    GenericHttpJsonV1,
    GovDatasetV1,
    CompanyRegistryV1,
    FilingRecordV1,
    PatentRecordV1,
    AcademicRecordV1,
    PricingTableV1,
}

impl StructuredSchemaId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::GenericHttpJsonV1 => "generic_http_json_v1",
            Self::GovDatasetV1 => "gov_dataset_v1",
            Self::CompanyRegistryV1 => "company_registry_v1",
            Self::FilingRecordV1 => "filing_record_v1",
            Self::PatentRecordV1 => "patent_record_v1",
            Self::AcademicRecordV1 => "academic_record_v1",
            Self::PricingTableV1 => "pricing_table_v1",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "generic_http_json_v1" => Some(Self::GenericHttpJsonV1),
            "gov_dataset_v1" => Some(Self::GovDatasetV1),
            "company_registry_v1" => Some(Self::CompanyRegistryV1),
            "filing_record_v1" => Some(Self::FilingRecordV1),
            "patent_record_v1" => Some(Self::PatentRecordV1),
            "academic_record_v1" => Some(Self::AcademicRecordV1),
            "pricing_table_v1" => Some(Self::PricingTableV1),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaRule {
    pub schema_id: StructuredSchemaId,
    pub required_attributes: &'static [&'static str],
}

pub fn schema_rule_for(schema_id: StructuredSchemaId) -> SchemaRule {
    match schema_id {
        StructuredSchemaId::GenericHttpJsonV1 => SchemaRule {
            schema_id,
            required_attributes: &[],
        },
        StructuredSchemaId::GovDatasetV1 => SchemaRule {
            schema_id,
            required_attributes: &["dataset_id", "value"],
        },
        StructuredSchemaId::CompanyRegistryV1 => SchemaRule {
            schema_id,
            required_attributes: &["registration_number", "jurisdiction", "status"],
        },
        StructuredSchemaId::FilingRecordV1 => SchemaRule {
            schema_id,
            required_attributes: &["filing_id", "filing_date"],
        },
        StructuredSchemaId::PatentRecordV1 => SchemaRule {
            schema_id,
            required_attributes: &["patent_number", "filing_date"],
        },
        StructuredSchemaId::AcademicRecordV1 => SchemaRule {
            schema_id,
            required_attributes: &["title", "publication_year"],
        },
        StructuredSchemaId::PricingTableV1 => SchemaRule {
            schema_id,
            required_attributes: &["product", "price"],
        },
    }
}
