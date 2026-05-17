pub mod r4;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::r4::{
        datetime::Date,
        generated::resources::{ClientApplication, Practitioner, Resource},
    };
    use haste_fhir_serialization_json::{FHIRJSONDeserializer, errors::DeserializeError};
    use haste_reflect::MetaValue;
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
            haste_fhir_serialization_json::to_string(
                &haste_fhir_serialization_json::from_str::<String>(k).unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn enum_resource_type_variant() {
        let resource = haste_fhir_serialization_json::from_str::<Resource>(
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

        let resource = haste_fhir_serialization_json::from_str::<Resource>(
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
            haste_fhir_serialization_json::to_string(resource.as_ref().unwrap()).unwrap()
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

        assert_eq!(
            haste_fhir_serialization_json::to_string(&address).unwrap(),
            serde_json::from_str::<Address>(address_string)
                .map(|a| haste_fhir_serialization_json::to_string(&a).unwrap())
                .unwrap()
        );

        let address_use: Option<String> = address.use_.unwrap().as_ref().into();
        assert_eq!(address_use.unwrap(), "home".to_string());
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
        assert!(matches!(
            serde_json::from_str::<Address>(address_string),
            Err(_)
        ));

        let address_string = r#"
        {
            "city": "Anytown",
            "_city": 5
        }
        "#;
        let address = Address::from_json_str(address_string);
        assert!(matches!(address, Err(DeserializeError::InvalidType(_))));

        assert!(matches!(
            serde_json::from_str::<Address>(address_string),
            Err(_)
        ));
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

        assert!(matches!(
            serde_json::from_str::<Address>(address_string),
            Err(_)
        ));
    }

    #[test]
    fn test_serialization_bundle() {
        let bundle_string = r#"
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

        let bundle1: r4::generated::resources::Bundle =
            r4::generated::resources::Bundle::from_json_str(bundle_string).unwrap();
        assert_eq!(bundle1.entry.as_ref().unwrap().len(), 2);
        let k = bundle1.entry.as_ref().unwrap()[0]
            .resource
            .as_ref()
            .unwrap()
            .typename();

        assert!(matches!(k, "MedicationRequest"));

        let bundle2 =
            serde_json::from_str::<r4::generated::resources::Bundle>(bundle_string).unwrap();

        assert!(matches!(
            bundle2.entry.as_ref().unwrap()[0]
                .resource
                .as_ref()
                .unwrap()
                .typename(),
            "MedicationRequest"
        ));

        assert_eq!(
            haste_fhir_serialization_json::to_string(&bundle1).unwrap(),
            haste_fhir_serialization_json::to_string(&bundle2).unwrap()
        );
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
            haste_fhir_serialization_json::to_string(patient.as_ref().unwrap()).unwrap(),
        );

        let patient2 = Patient::from_json_str(k).unwrap();
        assert_eq!(
            haste_fhir_serialization_json::to_string(&patient2).unwrap(),
            k
        );

        let patient2 = serde_json::from_str::<Patient>(k).unwrap();
        assert_eq!(
            haste_fhir_serialization_json::to_string(&patient2).unwrap(),
            k
        );
    }

    #[test]
    fn null_extension_many() {
        let patient_string = r#"
        {
            "resourceType": "Patient",
            "name": [
                {
                    "family": "Doe",
                    "given": ["John", "A."],
                    "_given": [null, {"id": "given-2"}],
                    "prefix": ["Mr."]
                }
            ]
        }"#;

        let patient = Patient::from_json_str(patient_string).unwrap();

        assert_eq!(
            patient.name.as_ref().unwrap()[0].given.as_ref().unwrap()[0]
                .value
                .as_ref()
                .unwrap(),
            "John"
        );

        assert_eq!(
            patient.name.as_ref().unwrap()[0].given.as_ref().unwrap()[0]
                .id
                .is_none(),
            true,
        );

        assert_eq!(
            patient.name.as_ref().unwrap()[0].given.as_ref().unwrap()[1]
                .id
                .as_ref()
                .unwrap(),
            "given-2",
        );

        assert_eq!(
            haste_fhir_serialization_json::to_string(&patient).unwrap(),
            "{\"resourceType\":\"Patient\",\"name\":[{\"family\":\"Doe\",\"_given\":[null,{\"id\":\"given-2\"}],\"given\":[\"John\",\"A.\"],\"prefix\":[\"Mr.\"]}]}"
        );

        let patient = serde_json::from_str::<Patient>(
            &haste_fhir_serialization_json::to_string(&patient).unwrap(),
        )
        .unwrap();

        assert_eq!(
            haste_fhir_serialization_json::to_string(&patient).unwrap(),
            "{\"resourceType\":\"Patient\",\"name\":[{\"family\":\"Doe\",\"_given\":[null,{\"id\":\"given-2\"}],\"given\":[\"John\",\"A.\"],\"prefix\":[\"Mr.\"]}]}"
        );
    }

    #[test]
    fn test_with_nulls_array_primitives() {
        let patient_string = r#"{
        "resourceType": "Patient",
        "name": [
          {
            "family": "Doe",
            "_given": [
              null,
              {
                "id": "given-2"
              }
            ],
            "given": [
              "John",
              null
            ],
            "prefix": [
              "Mr."
            ]
          }
        ]}"#;

        let patient = Patient::from_json_str(patient_string).unwrap();
        assert_eq!(
            haste_fhir_serialization_json::to_string(&patient).unwrap(),
            "{\"resourceType\":\"Patient\",\"name\":[{\"family\":\"Doe\",\"_given\":[null,{\"id\":\"given-2\"}],\"given\":[\"John\",null],\"prefix\":[\"Mr.\"]}]}"
        );

        let patient = serde_json::from_str::<Patient>(
            &haste_fhir_serialization_json::to_string(&patient).unwrap(),
        )
        .unwrap();

        assert_eq!(
            haste_fhir_serialization_json::to_string(&patient).unwrap(),
            "{\"resourceType\":\"Patient\",\"name\":[{\"family\":\"Doe\",\"_given\":[null,{\"id\":\"given-2\"}],\"given\":[\"John\",null],\"prefix\":[\"Mr.\"]}]}"
        );
    }

    #[test]
    fn test_serde_terminology() {
        use crate::r4::generated::terminology::AdministrativeGender;

        let admin_gender = serde_json::from_str::<AdministrativeGender>("\"male\"");

        assert!(matches!(admin_gender, Ok(AdministrativeGender::Male(None))));
    }

    #[test]
    fn test_serde_primitve() {
        use crate::r4::generated::types::{
            FHIRBoolean, FHIRDate, FHIRDecimal, FHIRInteger, FHIRPositiveInt, FHIRString,
        };

        let date = serde_json::from_str::<FHIRDate>("\"2020-01-01\"");

        assert!(matches!(
            date,
            Ok(FHIRDate {
                value: Some(Date::YearMonthDay(2020, 1, 1)),
                id: None,
                extension: None
            })
        ));

        let date = serde_json::from_str::<FHIRDate>("\"bad\"");

        assert!(matches!(date, Err(_)));

        let fhir_string = serde_json::from_str::<FHIRString>("\"hello\"");

        assert!(matches!(
            fhir_string,
            Ok(FHIRString {
                value: Some(_),
                id: None,
                extension: None
            })
        ));

        let fhir_boolean = serde_json::from_str::<FHIRBoolean>("true");

        assert!(matches!(
            fhir_boolean,
            Ok(FHIRBoolean {
                value: Some(true),
                id: None,
                extension: None
            })
        ));

        let fhir_boolean = serde_json::from_str::<FHIRBoolean>("false");

        assert!(matches!(
            fhir_boolean,
            Ok(FHIRBoolean {
                value: Some(false),
                id: None,
                extension: None
            })
        ));

        let fhir_integer = serde_json::from_str::<FHIRInteger>("42");
        assert!(matches!(
            fhir_integer,
            Ok(FHIRInteger {
                value: Some(42),
                id: None,
                extension: None
            })
        ));

        let fhir_positive_int = serde_json::from_str::<FHIRPositiveInt>("42");
        assert!(matches!(
            fhir_positive_int,
            Ok(FHIRPositiveInt {
                value: Some(42),
                id: None,
                extension: None
            })
        ));

        let invalid_fhir_positive_int = serde_json::from_str::<FHIRPositiveInt>("42.5");
        assert!(matches!(invalid_fhir_positive_int, Err(_)));

        let invalid_positive_int = serde_json::from_str::<FHIRPositiveInt>("-1");
        assert!(matches!(invalid_positive_int, Err(_)));

        let fhir_decimal = serde_json::from_str::<FHIRDecimal>("3.14");
        assert!(matches!(
            fhir_decimal,
            Ok(FHIRDecimal {
                value: Some(3.14),
                id: None,
                extension: None
            })
        ));
    }

    #[test]
    fn test_cardinality() {
        let client_application_string = r#"{
	    "id": "cli",
        "resourceType": "ClientApplication",
        "name": "CLI",
        "grantType": ["authorization_code"],
        "responseTypes": "token",
        "secret": "testing",
        "redirectUri": [
          "http://localhost:8080/1", 
          "http://localhost:8080/2", 
          "http://localhost:8080/3", 
          "http://localhost:8080/4", 
          "http://localhost:8080/5"],
        "scope": "openid system/*.*"
      }"#;

        let client_app = serde_json::from_str::<ClientApplication>(&client_application_string);

        println!("{:#?}", client_app);

        assert!(client_app.is_ok());

        let client_application_string = r#"{
	    "id": "cli",
        "resourceType": "ClientApplication",
        "name": "CLI",
        "grantType": ["authorization_code"],
        "responseTypes": "token",
        "secret": "testing",
        "redirectUri": [
          "http://localhost:8080/1", 
          "http://localhost:8080/2", 
          "http://localhost:8080/3", 
          "http://localhost:8080/4", 
          "http://localhost:8080/5", 
          "http://localhost:8080/6"],
        "scope": "openid system/*.*"
      }"#;

        let client_app = serde_json::from_str::<ClientApplication>(&client_application_string);

        assert!(client_app.is_err());
    }
}
