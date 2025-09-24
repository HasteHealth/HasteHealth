pub mod r4;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r4::generated::resources::{Practitioner, Resource};
    use oxidized_fhir_serialization_json::{FHIRJSONDeserializer, errors::DeserializeError};
    use oxidized_reflect::MetaValue;
    use r4::generated::{resources::Patient, types::Address};
    use serde_json;

    #[test]
    fn test_enum_with_extension() {
        let term_ = r4::generated::terminology::AdministrativeGender::Male(Some(
            r4::generated::types::Element {
                id: Some("test".to_string()),
                ..r4::generated::types::Element::default()
            },
        ));
        assert_eq!(term_.typename(), "FHIRCode");
        let k = term_
            .get_field("value")
            .unwrap()
            .as_any()
            .downcast_ref::<String>()
            .unwrap();
        assert_eq!(k, &"male");
    }

    #[test]
    fn test_serializing_string_html() {
        let k = r#""<div xmlns=\"http://www.w3.org/1999/xhtml\">\n      <p>Dr Adam Careful is a Referring Practitioner for Acme Hospital from 1-Jan 2012 to 31-Mar\n        2012</p>\n    </div>""#;
        let parsed_str_serde =
            serde_json::to_string(&serde_json::from_str::<serde_json::Value>(k).unwrap()).unwrap();

        assert_eq!(
            parsed_str_serde,
            oxidized_fhir_serialization_json::to_string(
                &oxidized_fhir_serialization_json::from_str::<String>(k).unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn enum_resource_type_variant() {
        let resource = oxidized_fhir_serialization_json::from_str::<Resource>(
            r#"{
            "resourceType": "Patient",
            "address": [
                {
                    "use": "home",
                    "line": ["123 Main St"],
                    "_line": [{"id": "hello-world"}],
                    "city": "Anytown",
                    "_city": {
                        "id": "city-id"
                    },
                    "state": "CA",
                    "postalCode": "12345"
                }]
            
            }"#,
        );

        assert!(matches!(resource, Ok(Resource::Patient(Patient { .. }))));

        let resource = oxidized_fhir_serialization_json::from_str::<Resource>(
            r#"{
  "resourceType": "Practitioner",
  "id": "example",
  "text": {
    "status": "generated",
    "div": "<div xmlns=\"http://www.w3.org/1999/xhtml\">\n      <p>Dr Adam Careful is a Referring Practitioner for Acme Hospital from 1-Jan 2012 to 31-Mar\n        2012</p>\n    </div>"
  },
  "identifier": [
    {
      "system": "http://www.acme.org/practitioners",
      "value": "23"
    }
  ],
  "active": true,
  "name": [
    {
      "family": "Careful",
      "given": [
        "Adam"
      ],
      "prefix": [
        "Dr"
      ]
    }
  ],
  "address": [
    {
      "use": "home",
      "line": [
        "534 Erewhon St"
      ],
      "city": "PleasantVille",
      "state": "Vic",
      "postalCode": "3999"
    }
  ],
  "qualification": [
    {
      "identifier": [
        {
          "system": "http://example.org/UniversityIdentifier",
          "value": "12345"
        }
      ],
      "code": {
        "coding": [
          {
            "system": "http://terminology.hl7.org/CodeSystem/v2-0360/2.7",
            "code": "BS",
            "display": "Bachelor of Science"
          }
        ],
        "text": "Bachelor of Science"
      },
      "period": {
        "start": "1995"
      },
      "issuer": {
        "display": "Example University"
      }
    }
  ]
}"#,
        );

        assert!(matches!(
            resource,
            Ok(Resource::Practitioner(Practitioner { .. }))
        ));

        assert_eq!(
            "{\"resourceType\":\"Practitioner\",\"id\":\"example\",\"text\":{\"status\":\"generated\",\"div\":\"<div xmlns=\\\"http://www.w3.org/1999/xhtml\\\">\\n      <p>Dr Adam Careful is a Referring Practitioner for Acme Hospital from 1-Jan 2012 to 31-Mar\\n        2012</p>\\n    </div>\"},\"identifier\":[{\"system\":\"http://www.acme.org/practitioners\",\"value\":\"23\"}],\"active\":true,\"name\":[{\"family\":\"Careful\",\"given\":[\"Adam\"],\"prefix\":[\"Dr\"]}],\"address\":[{\"use\":\"home\",\"line\":[\"534 Erewhon St\"],\"city\":\"PleasantVille\",\"state\":\"Vic\",\"postalCode\":\"3999\"}],\"qualification\":[{\"identifier\":[{\"system\":\"http://example.org/UniversityIdentifier\",\"value\":\"12345\"}],\"code\":{\"coding\":[{\"system\":\"http://terminology.hl7.org/CodeSystem/v2-0360/2.7\",\"code\":\"BS\",\"display\":\"Bachelor of Science\"}],\"text\":\"Bachelor of Science\"},\"period\":{\"start\":\"1995\"},\"issuer\":{\"display\":\"Example University\"}}]}",
            oxidized_fhir_serialization_json::to_string(resource.as_ref().unwrap()).unwrap()
        );
    }

    #[test]
    fn test_valid_address_with_extensions() {
        let address_string = r#"
        {
            "use": "home",
            "line": ["123 Main St"],
            "_line": [{"id": "hello-world"}],
            "city": "Anytown",
            "_city": {
                "id": "city-id"
            },
            "state": "CA",
            "postalCode": "12345"
        }
        "#;
        let address: Address = Address::from_json_str(address_string).unwrap();

        assert_eq!(address.use_.unwrap().value.unwrap(), "home".to_string());
        assert_eq!(
            address.line.as_ref().unwrap()[0].value.as_ref().unwrap(),
            &"123 Main St".to_string()
        );
        assert_eq!(
            address.line.as_ref().unwrap()[0].id.as_ref().unwrap(),
            &"hello-world".to_string()
        );
        assert_eq!(
            address.city.as_ref().unwrap().value.as_ref().unwrap(),
            &"Anytown".to_string()
        );
        assert_eq!(address.state.unwrap().value.unwrap(), "CA".to_string());
        assert_eq!(
            address.postalCode.unwrap().value.unwrap(),
            "12345".to_string()
        );
        assert_eq!(
            address.city.as_ref().unwrap().id.as_ref().unwrap(),
            &"city-id".to_string()
        );
    }

    #[test]
    fn test_invalid_address_with_extensions() {
        let address_string = r#"
        {
            "line": ["123 Main St"],
            "_line": {"id": "hello-world"}
        }
        "#;
        let address = Address::from_json_str(address_string);
        assert!(matches!(address, Err(DeserializeError::InvalidType(_))));

        let address_string = r#"
        {
            "city": "Anytown",
            "_city": 5
        }
        "#;
        let address = Address::from_json_str(address_string);
        assert!(matches!(address, Err(DeserializeError::InvalidType(_))));
    }

    #[test]
    fn test_invalid_fields() {
        let address_string = r#"
        {
            "line": ["123 Main St"],
            "_line": [{"id": "hello-world"}],
            "bad_field": "This should not be here"
        }
        "#;

        let address = Address::from_json_str(address_string);

        assert_eq!(
            address.unwrap_err().to_string(),
            "Unknown field encountered: Address: 'bad_field'"
        );
    }

    #[test]
    fn test_serialization_bundle() {
        let bundle = r#"
        {
  "resourceType": "Bundle",
  "id": "bundle-example",
  "meta": {
    "lastUpdated": "2014-08-18T01:43:30Z"
  },
  "type": "searchset",
  "total": 3,
  "link": [
    {
      "relation": "self",
      "url": "https://example.com/base/MedicationRequest?patient=347&_include=MedicationRequest.medication&_count=2"
    },
    {
      "relation": "next",
      "url": "https://example.com/base/MedicationRequest?patient=347&searchId=ff15fd40-ff71-4b48-b366-09c706bed9d0&page=2"
    }
  ],
  "entry": [
    {
      "fullUrl": "https://example.com/base/MedicationRequest/3123",
      "resource": {
        "resourceType": "MedicationRequest",
        "id": "3123",
        "text": {
          "status": "generated",
          "div": "<div xmlns=\"http://www.w3.org/1999/xhtml\"><p><b>Generated Narrative with Details</b></p><p><b>id</b>: 3123</p><p><b>status</b>: unknown</p><p><b>intent</b>: order</p><p><b>medication</b>: <a>Medication/example</a></p><p><b>subject</b>: <a>Patient/347</a></p></div>"
        },
        "status": "unknown",
        "intent": "order",
        "medicationReference": {
          "reference": "Medication/example"
        },
        "subject": {
          "reference": "Patient/347"
        }
      },
      "search": {
        "mode": "match",
        "score": 1
      }
    },
    {
      "fullUrl": "https://example.com/base/Medication/example",
      "resource": {
        "resourceType": "Medication",
        "id": "example",
        "text": {
          "status": "generated",
          "div": "<div xmlns=\"http://www.w3.org/1999/xhtml\"><p><b>Generated Narrative with Details</b></p><p><b>id</b>: example</p></div>"
        }
      },
      "search": {
        "mode": "include"
      }
    }
  ]
}
        "#;

        let bundle: r4::generated::resources::Bundle =
            r4::generated::resources::Bundle::from_json_str(bundle).unwrap();
        assert_eq!(bundle.entry.as_ref().unwrap().len(), 2);
        let k = bundle.entry.as_ref().unwrap()[0]
            .resource
            .as_ref()
            .unwrap()
            .typename();

        assert!(matches!(k, "MedicationRequest"));
    }

    #[test]
    fn test_patient_resource() {
        let patient_string = r#"
        {
            "resourceType": "Patient",
            "address": [
                {
                    "use": "home",
                    "line": ["123 Main St"],
                    "_line": [{"id": "hello-world"}],
                    "city": "Anytown",
                    "_city": {
                        "id": "city-id"
                    },
                    "state": "CA",
                    "postalCode": "12345"
                },
                {
                    "use": "home",
                    "line": ["123 Main St"],
                    "_line": [{"id": "hello-world"}],
                    "city": "Anytown",
                    "_city": {
                        "id": "city-id"
                    },
                    "state": "CA",
                    "postalCode": "12345"
                },
                {
                    "use": "home",
                    "line": ["123 Main St"],
                    "_line": [{"id": "hello-world"}],
                    "city": "Anytown",
                    "_city": {
                        "id": "city-id"
                    },
                    "state": "CA",
                    "postalCode": "12345"
                },
                {
                    "use": "home",
                    "line": ["123 Main St"],
                    "_line": [{"id": "hello-world"}],
                    "city": "Anytown",
                    "_city": {
                        "id": "city-id"
                    },
                    "state": "CA",
                    "postalCode": "12345"
                },
                {
                    "use": "home",
                    "line": ["123 Main St"],
                    "_line": [{"id": "hello-world"}],
                    "city": "Anytown",
                    "_city": {
                        "id": "city-id"
                    },
                    "state": "CA",
                    "postalCode": "12345"
                }

            ]
        }
        "#
        .trim();

        let patient = Patient::from_json_str(patient_string);

        assert!(matches!(patient, Ok(Patient { .. })));
        assert_eq!(patient.as_ref().unwrap().address.as_ref().unwrap().len(), 5);

        assert_eq!(
            patient.as_ref().unwrap().address.as_ref().unwrap()[0]
                .city
                .as_ref()
                .unwrap()
                .value
                .as_ref()
                .unwrap(),
            "Anytown"
        );

        let k = "{\"resourceType\":\"Patient\",\"address\":[{\"use\":\"home\",\"_line\":[{\"id\":\"hello-world\"}],\"line\":[\"123 Main St\"],\"city\":\"Anytown\",\"_city\":{\"id\":\"city-id\"},\"state\":\"CA\",\"postalCode\":\"12345\"},{\"use\":\"home\",\"_line\":[{\"id\":\"hello-world\"}],\"line\":[\"123 Main St\"],\"city\":\"Anytown\",\"_city\":{\"id\":\"city-id\"},\"state\":\"CA\",\"postalCode\":\"12345\"},{\"use\":\"home\",\"_line\":[{\"id\":\"hello-world\"}],\"line\":[\"123 Main St\"],\"city\":\"Anytown\",\"_city\":{\"id\":\"city-id\"},\"state\":\"CA\",\"postalCode\":\"12345\"},{\"use\":\"home\",\"_line\":[{\"id\":\"hello-world\"}],\"line\":[\"123 Main St\"],\"city\":\"Anytown\",\"_city\":{\"id\":\"city-id\"},\"state\":\"CA\",\"postalCode\":\"12345\"},{\"use\":\"home\",\"_line\":[{\"id\":\"hello-world\"}],\"line\":[\"123 Main St\"],\"city\":\"Anytown\",\"_city\":{\"id\":\"city-id\"},\"state\":\"CA\",\"postalCode\":\"12345\"}]}";

        assert_eq!(
            k,
            oxidized_fhir_serialization_json::to_string(patient.as_ref().unwrap()).unwrap(),
        );

        let patient2 = Patient::from_json_str(k).unwrap();
        assert_eq!(
            oxidized_fhir_serialization_json::to_string(&patient2).unwrap(),
            k
        );
    }
}
