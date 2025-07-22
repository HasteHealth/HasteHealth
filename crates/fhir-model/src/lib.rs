#![feature(test)]
pub mod r4;

#[cfg(test)]
mod tests {
    extern crate test;

    use super::*;
    use crate::r4::types::{Practitioner, Resource};
    use fhir_serialization_json::{FHIRJSONDeserializer, errors::DeserializeError};
    use r4::types::{Address, Patient};
    use reflect::MetaValue;
    use serde_json;
    use test::Bencher;

    #[test]
    fn test_serializing_string_html() {
        let k = r#""<div xmlns=\"http://www.w3.org/1999/xhtml\">\n      <p>Dr Adam Careful is a Referring Practitioner for Acme Hospital from 1-Jan 2012 to 31-Mar\n        2012</p>\n    </div>""#;
        let parsed_str_serde =
            serde_json::to_string(&serde_json::from_str::<serde_json::Value>(k).unwrap()).unwrap();

        assert_eq!(
            parsed_str_serde,
            fhir_serialization_json::to_string(
                &fhir_serialization_json::from_str::<String>(k).unwrap()
            )
            .unwrap()
        );
    }

    #[test]
    fn enum_resource_type_variant() {
        let resource = fhir_serialization_json::from_str::<Resource>(
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

        let resource = fhir_serialization_json::from_str::<Resource>(
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
            fhir_serialization_json::to_string(resource.as_ref().unwrap()).unwrap()
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
            "Unknown field encountered: Address: bad_field"
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

        let bundle: r4::types::Bundle = r4::types::Bundle::from_json_str(bundle).unwrap();
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
            fhir_serialization_json::to_string(patient.as_ref().unwrap()).unwrap(),
        );

        let patient2 = Patient::from_json_str(k).unwrap();
        assert_eq!(fhir_serialization_json::to_string(&patient2).unwrap(), k);
    }

    #[bench]
    fn complex_patient(b: &mut Bencher) {
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
        }"#;

        b.iter(|| Patient::from_json_str(patient_string).unwrap())
    }

    #[bench]
    fn raw_json_complex_patient(b: &mut Bencher) {
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
        }"#;

        b.iter(|| serde_json::from_str::<serde_json::Value>(patient_string).unwrap());
    }

    // #[bench]
    // fn serde_patient_direct(b: &mut Bencher) {
    //     let patient_string = r#"{
    //         "resourceType": "Patient",
    //         "address": [
    //             {
    //                 "use": {"value": "home"},
    //                 "line": [{"id": "hello-world", "value": "123 Main St"}],
    //                 "city": {"id": "city-id", "value": "Anytown"},
    //                 "state": {"value": "CA"},
    //                 "postalCode": {"value": "12345"}
    //             },
    //             {
    //                 "use": {"value": "home"},
    //                 "line": [{"id": "hello-world", "value": "123 Main St"}],
    //                 "city": {"id": "city-id", "value": "Anytown"},
    //                 "state": {"value": "CA"},
    //                 "postalCode": {"value": "12345"}
    //             },
    //             {
    //                 "use": {"value": "home"},
    //                 "line": [{"id": "hello-world", "value": "123 Main St"}],
    //                 "city": {"id": "city-id", "value": "Anytown"},
    //                 "state": {"value": "CA"},
    //                 "postalCode": {"value": "12345"}
    //             },
    //             {
    //                 "use": {"value": "home"},
    //                 "line": [{"id": "hello-world", "value": "123 Main St"}],
    //                 "city": {"id": "city-id", "value": "Anytown"},
    //                 "state": {"value": "CA"},
    //                 "postalCode": {"value": "12345"}
    //             },
    //             {
    //                 "use": {"value": "home"},
    //                 "line": [{"id": "hello-world", "value": "123 Main St"}],
    //                 "city": {"id": "city-id", "value": "Anytown"},
    //                 "state": {"value": "CA"},
    //                 "postalCode": {"value": "12345"}
    //             },
    //             {
    //                 "use": {"value": "home"},
    //                 "line": [{"id": "hello-world", "value": "123 Main St"}],
    //                 "city": {"id": "city-id", "value": "Anytown"},
    //                 "state": {"value": "CA"},
    //                 "postalCode": {"value": "12345"}
    //             },
    //             {
    //                 "use": {"value": "home"},
    //                 "line": [{"id": "hello-world", "value": "123 Main St"}],
    //                 "city": {"id": "city-id", "value": "Anytown"},
    //                 "state": {"value": "CA"},
    //                 "postalCode": {"value": "12345"}
    //             },
    //             {
    //                 "use": {"value": "home"},
    //                 "line": [{"id": "hello-world", "value": "123 Main St"}],
    //                 "city": {"id": "city-id", "value": "Anytown"},
    //                 "state": {"value": "CA"},
    //                 "postalCode": {"value": "12345"}
    //             },
    //             {
    //                 "use": {"value": "home"},
    //                 "line": [{"id": "hello-world", "value": "123 Main St"}],
    //                 "city": {"id": "city-id", "value": "Anytown"},
    //                 "state": {"value": "CA"},
    //                 "postalCode": {"value": "12345"}
    //             },
    //             {
    //                 "use": {"value": "home"},
    //                 "line": [{"id": "hello-world", "value": "123 Main St"}],
    //                 "city": {"id": "city-id", "value": "Anytown"},
    //                 "state": {"value": "CA"},
    //                 "postalCode": {"value": "12345"}
    //             }
    //         ]
    //     }"#;

    //     b.iter(|| serde_json::from_str::<Patient>(patient_string).unwrap());
    // }

    #[bench]
    fn hl7_general_patient_example(b: &mut Bencher) {
        let patient_string = r#"
            {
                "resourceType": "Patient",
                "id": "example",
                "text": {
                    "status": "generated",
                    "div": "<div xmlns=\"http://www.w3.org/1999/xhtml\">\n\t\t\t<table>\n\t\t\t\t<tbody>\n\t\t\t\t\t<tr>\n\t\t\t\t\t\t<td>Name</td>\n\t\t\t\t\t\t<td>Peter James \n              <b>Chalmers</b> (&quot;Jim&quot;)\n            </td>\n\t\t\t\t\t</tr>\n\t\t\t\t\t<tr>\n\t\t\t\t\t\t<td>Address</td>\n\t\t\t\t\t\t<td>534 Erewhon, Pleasantville, Vic, 3999</td>\n\t\t\t\t\t</tr>\n\t\t\t\t\t<tr>\n\t\t\t\t\t\t<td>Contacts</td>\n\t\t\t\t\t\t<td>Home: unknown. Work: (03) 5555 6473</td>\n\t\t\t\t\t</tr>\n\t\t\t\t\t<tr>\n\t\t\t\t\t\t<td>Id</td>\n\t\t\t\t\t\t<td>MRN: 12345 (Acme Healthcare)</td>\n\t\t\t\t\t</tr>\n\t\t\t\t</tbody>\n\t\t\t</table>\n\t\t</div>"
                },
                "identifier": [
                    {
                    "use": "usual",
                    "type": {
                        "coding": [
                        {
                            "system": "http://terminology.hl7.org/CodeSystem/v2-0203",
                            "code": "MR"
                        }
                        ]
                    },
                    "system": "urn:oid:1.2.36.146.595.217.0.1",
                    "value": "12345",
                    "period": {
                        "start": "2001-05-06"
                    },
                    "assigner": {
                        "display": "Acme Healthcare"
                    }
                    }
                ],
                "active": true,
                "name": [
                    {
                    "use": "official",
                    "family": "Chalmers",
                    "given": [
                        "Peter",
                        "James"
                    ]
                    },
                    {
                    "use": "usual",
                    "given": [
                        "Jim"
                    ]
                    },
                    {
                    "use": "maiden",
                    "family": "Windsor",
                    "given": [
                        "Peter",
                        "James"
                    ],
                    "period": {
                        "end": "2002"
                    }
                    }
                ],
                "telecom": [
                    {
                    "use": "home"
                    },
                    {
                    "system": "phone",
                    "value": "(03) 5555 6473",
                    "use": "work",
                    "rank": 1
                    },
                    {
                    "system": "phone",
                    "value": "(03) 3410 5613",
                    "use": "mobile",
                    "rank": 2
                    },
                    {
                    "system": "phone",
                    "value": "(03) 5555 8834",
                    "use": "old",
                    "period": {
                        "end": "2014"
                    }
                    }
                ],
                "gender": "male",
                "birthDate": "1974-12-25",
                "_birthDate": {
                    "extension": [
                    {
                        "url": "http://hl7.org/fhir/StructureDefinition/patient-birthTime",
                        "valueDateTime": "1974-12-25T14:35:45-05:00"
                    }
                    ]
                },
                "deceasedBoolean": false,
                "address": [
                    {
                    "use": "home",
                    "type": "both",
                    "text": "534 Erewhon St PeasantVille, Rainbow, Vic  3999",
                    "line": [
                        "534 Erewhon St"
                    ],
                    "city": "PleasantVille",
                    "district": "Rainbow",
                    "state": "Vic",
                    "postalCode": "3999",
                    "period": {
                        "start": "1974-12-25"
                    }
                    }
                ],
                "contact": [
                    {
                    "relationship": [
                        {
                        "coding": [
                            {
                            "system": "http://terminology.hl7.org/CodeSystem/v2-0131",
                            "code": "N"
                            }
                        ]
                        }
                    ],
                    "name": {
                        "family": "du Marché",
                        "_family": {
                        "extension": [
                            {
                            "url": "http://hl7.org/fhir/StructureDefinition/humanname-own-prefix",
                            "valueString": "VV"
                            }
                        ]
                        },
                        "given": [
                        "Bénédicte"
                        ]
                    },
                    "telecom": [
                        {
                        "system": "phone",
                        "value": "+33 (237) 998327"
                        }
                    ],
                    "address": {
                        "use": "home",
                        "type": "both",
                        "line": [
                        "534 Erewhon St"
                        ],
                        "city": "PleasantVille",
                        "district": "Rainbow",
                        "state": "Vic",
                        "postalCode": "3999",
                        "period": {
                        "start": "1974-12-25"
                        }
                    },
                    "gender": "female",
                    "period": {
                        "start": "2012"
                    }
                    }
                ],
                "managingOrganization": {
                    "reference": "Organization/1"
                }
            }
        "#;

        b.iter(|| Patient::from_json_str(patient_string).unwrap());
    }
}
