use std::path::Path;

use crate::utilities::load;
/// Generate typescripts for testscript test cases by providing resource_files.
use haste_fhir_model::r4::generated::{
    resources::{
        Resource, TestScript, TestScriptFixture, TestScriptSetupActionAssert,
        TestScriptSetupActionOperation, TestScriptTeardown, TestScriptTeardownAction,
        TestScriptTest, TestScriptTestAction,
    },
    terminology::{AssertDirectionCodes, DefinedTypes, PublicationStatus},
    types::{Coding, FHIRBoolean, FHIRCode, FHIRId, FHIRString, FHIRUri, Meta, Reference},
};
use haste_reflect::MetaValue;
use walkdir::WalkDir;

fn file_path_to_resources(file_path: &Path) -> Result<Vec<Box<Resource>>, String> {
    let resource = load::load_from_file(file_path)?;

    Ok(match resource {
        Resource::Bundle(bundle) => bundle
            .entry
            .unwrap_or(vec![])
            .into_iter()
            .filter_map(|entry| entry.resource)
            .collect::<Vec<_>>(),
        _ => vec![Box::new(resource)],
    })
}

fn get_meta_mutable<'a>(resource: &'a mut Resource) -> Result<&'a mut Meta, String> {
    let meta: &mut dyn std::any::Any = resource
        .get_field_mut("meta")
        .ok_or("Missing Meta Field".to_string())?;
    let meta: &mut Option<Box<Meta>> = meta
        .downcast_mut::<Option<Box<Meta>>>()
        .ok_or("Failed to downcast meta".to_string())?;

    if meta.is_none() {
        *meta = Some(Box::new(Meta::default()))
    }

    Ok(meta.as_mut().unwrap())
}

fn set_resource_tag(tag: &str, resource: &mut Resource) -> Result<(), String> {
    let meta = get_meta_mutable(resource)?;

    meta.tag = Some(vec![Box::new(Coding {
        code: Some(Box::new(FHIRCode {
            value: Some(tag.to_string()),
            ..Default::default()
        })),
        ..Default::default()
    })]);

    Ok(())
}

fn set_resource_id(id: &str, resource: &mut Resource) -> Result<(), String> {
    let id_field: &mut dyn std::any::Any = resource
        .get_field_mut("id")
        .ok_or("Missing id field".to_string())?;

    let id_field: &mut Option<String> = id_field
        .downcast_mut::<Option<String>>()
        .ok_or("Failed to downcast id field".to_string())?;

    *id_field = Some(id.to_string());

    Ok(())
}

fn fixture_name(i: usize, resource_type: &str) -> String {
    format!("fixture-{}-{}", resource_type, i)
}

fn generate_testcases_for_resource(
    tag: &str,
    index: usize,
    resource: &Resource,
) -> Vec<TestScriptTest> {
    let resource_type = resource.resource_type();
    let defined_type = Some(Box::new(
        DefinedTypes::try_from(resource_type.as_ref().to_string())
            .expect("Unsupported resource type"),
    ));

    vec![TestScriptTest {
        name: Some(Box::new(FHIRString {
            value: Some(format!("Test for resource with tag: {}", tag)),
            ..Default::default()
        })),
        action: vec![
            TestScriptTestAction {
                operation: Some(TestScriptSetupActionOperation {
                    type_: Some(Box::new(Coding {
                        system: Some(Box::new(FHIRUri {
                            value: Some(
                                "http://terminology.hl7.org/CodeSystem/testscript-operation-codes"
                                    .to_string(),
                            ),
                            ..Default::default()
                        })),
                        code: Some(Box::new(FHIRCode {
                            value: Some("create".to_string()),
                            ..Default::default()
                        })),
                        ..Default::default()
                    })),
                    resource: defined_type.clone(),
                    sourceId: Some(Box::new(FHIRId {
                        value: Some(fixture_name(index, resource_type.as_ref()).to_string()),
                        ..Default::default()
                    })),
                    responseId: Some(Box::new(FHIRId {
                        value: Some(fixture_name(index, resource_type.as_ref())),
                        ..Default::default()
                    })),
                    encodeRequestUrl: Box::new(FHIRBoolean {
                        value: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            TestScriptTestAction {
                assert: Some(TestScriptSetupActionAssert {
                    label: Some(Box::new(FHIRString {
                        value: Some("Read created resource".to_string()),
                        ..Default::default()
                    })),
                    description: Some(Box::new(FHIRString {
                        value: Some(format!(
                            "Confirm resource of type {} created.",
                            resource_type.as_ref()
                        )),
                        ..Default::default()
                    })),
                    direction: Some(Box::new(AssertDirectionCodes::Response(None))),
                    resource: defined_type.clone(),
                    warningOnly: Box::new(FHIRBoolean {
                        value: Some(false),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
        ],
        ..Default::default()
    }]
}

fn generate_fixtures_for_resource(
    testscript: &mut TestScript,
    resources: Vec<Box<Resource>>,
) -> Result<(), String> {
    let mut contained = vec![];
    let mut fixtures = vec![];

    for (index, resource) in resources.into_iter().enumerate() {
        let resource_type = resource.resource_type();
        let fixture_id = fixture_name(index, resource_type.as_ref());

        fixtures.push(TestScriptFixture {
            id: Some(fixture_id.clone()),
            autocreate: Box::new(FHIRBoolean {
                value: Some(false),
                ..Default::default()
            }),
            autodelete: Box::new(FHIRBoolean {
                value: Some(false),
                ..Default::default()
            }),
            resource: Some(Box::new(Reference {
                reference: Some(Box::new(FHIRString {
                    value: Some(format!("#{}", fixture_id)),
                    ..Default::default()
                })),
                ..Default::default()
            })),
            ..Default::default()
        });
        contained.push(resource);
    }

    testscript.contained = Some(contained);
    testscript.fixture = Some(fixtures);

    Ok(())
}

fn create_tag(file_path: &Path) -> String {
    file_path
        .to_str()
        .unwrap()
        .replace("/", "-")
        .replace("\\", "-")
        .replace(".", "-")
}

fn generate_testscript_from_file(file_path: &Path) -> Result<TestScript, String> {
    let mut testscript = TestScript::default();
    let mut resources = file_path_to_resources(file_path)?;

    let tag = create_tag(file_path);

    testscript.url = Box::new(FHIRUri {
        value: Some(tag.to_string()),
        ..Default::default()
    });
    testscript.status = Box::new(PublicationStatus::Active(None));
    testscript.id = Some(tag.to_string());
    testscript.name = Box::new(FHIRString {
        value: Some(tag.to_string()),
        ..Default::default()
    });

    for (i, resource) in resources.iter_mut().enumerate() {
        set_resource_tag(&tag, resource).expect("Failed to set resource tag");
        set_resource_id(
            &fixture_name(i, &resource.resource_type().as_ref()),
            resource,
        )
        .expect("Failed to set resource id");
    }

    generate_fixtures_for_resource(&mut testscript, resources.clone())?;

    testscript.test = Some(
        resources
            .iter()
            .enumerate()
            .map(|(i, r)| generate_testcases_for_resource(&tag, i, r))
            .flatten()
            .collect::<Vec<_>>(),
    );

    testscript.teardown = Some(TestScriptTeardown {
        action: vec![TestScriptTeardownAction {
            operation: TestScriptSetupActionOperation {
                type_: Some(Box::new(Coding {
                    system: Some(Box::new(FHIRUri {
                        value: Some(
                            "http://terminology.hl7.org/CodeSystem/testscript-operation-codes"
                                .to_string(),
                        ),
                        ..Default::default()
                    })),
                    code: Some(Box::new(FHIRCode {
                        value: Some("delete".to_string()),
                        ..Default::default()
                    })),
                    ..Default::default()
                })),
                encodeRequestUrl: Box::new(FHIRBoolean {
                    value: Some(true),
                    ..Default::default()
                }),
                resource: None,
                params: Some(Box::new(FHIRString {
                    value: Some(format!("_tag={}", tag)),
                    ..Default::default()
                })),
                description: Some(Box::new(FHIRString {
                    value: Some("Delete resources created in test.".to_string()),
                    ..Default::default()
                })),

                ..Default::default()
            },
            ..Default::default()
        }],
        ..Default::default()
    });

    Ok(testscript)
}

pub fn generate_testscripts(file_paths: &Vec<String>) -> Result<Vec<TestScript>, String> {
    let mut testscripts = vec![];
    for dir_path in file_paths {
        let walker = WalkDir::new(dir_path).into_iter();
        for entry in walker
            .filter_map(|e| e.ok())
            .filter(|e| e.metadata().unwrap().is_file())
        {
            let testscript = generate_testscript_from_file(&entry.path().to_path_buf())?;
            testscripts.push(testscript);
        }
    }

    Ok(testscripts)
}
