use criterion::{Criterion, criterion_group, criterion_main};
use haste_fhir_model::r4::generated::resources::Patient;
use haste_fhir_serialization_json::FHIRJSONDeserializer;

fn complex_patient(c: &mut Criterion) {
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

    c.bench_function("complex_patient", |b| {
        b.iter(|| Patient::from_json_str(patient_string).unwrap())
    });
}

fn raw_json_complex_patient(c: &mut Criterion) {
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

    c.bench_function("raw_json_complex_patient", |b| {
        b.iter(|| serde_json::from_str::<serde_json::Value>(patient_string).unwrap())
    });
}

fn hl7_general_patient_example(c: &mut Criterion) {
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

    c.bench_function("hl7_general_patient_example", |b| {
        b.iter(|| Patient::from_json_str(patient_string).unwrap())
    });
}
criterion_group!(
    benches,
    raw_json_complex_patient,
    hl7_general_patient_example,
    complex_patient
);
criterion_main!(benches);
