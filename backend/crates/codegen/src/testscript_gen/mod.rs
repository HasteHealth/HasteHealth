use std::path::Path;

use crate::utilities::load;
/// Generate typescripts for testscript test cases by providing resource_files.
use haste_fhir_model::r4::generated::{
    resources::{
        Resource, TestScript, TestScriptFixture, TestScriptSetupActionAssert,
        TestScriptSetupActionOperation, TestScriptTeardown, TestScriptTeardownAction,
        TestScriptTest, TestScriptTestAction,
    },
    terminology::{AssertDirectionCodes, BoundCode, DefinedTypes, PublicationStatus},
    types::{Coding, Meta, Reference},
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
        code: Some(Box::new(tag.to_string().into())),
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
    let defined_type = Some(
        BoundCode::<DefinedTypes>::new(resource_type.as_ref()).expect("Unsupported resource type"),
    );

    vec![TestScriptTest {
        name: Some(Box::new(
            format!("Test for resource with tag: {}", tag).into(),
        )),
        action: vec![
            TestScriptTestAction {
                operation: Some(TestScriptSetupActionOperation {
                    type_: Some(Box::new(Coding {
                        system: Some(Box::new(
                            "http://terminology.hl7.org/CodeSystem/testscript-operation-codes"
                                .to_string()
                                .into(),
                        )),
                        code: Some(Box::new("create".to_string().into())),
                        ..Default::default()
                    })),
                    resource: defined_type.clone(),
                    sourceId: Some(Box::new(
                        fixture_name(index, resource_type.as_ref())
                            .to_string()
                            .into(),
                    )),
                    responseId: Some(Box::new(fixture_name(index, resource_type.as_ref()).into())),
                    encodeRequestUrl: Box::new(true.into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            TestScriptTestAction {
                assert: Some(TestScriptSetupActionAssert {
                    label: Some(Box::new("Read created resource".to_string().into())),
                    description: Some(Box::new(
                        format!(
                            "Confirm resource of type {} created.",
                            resource_type.as_ref()
                        )
                        .into(),
                    )),
                    direction: Some(AssertDirectionCodes::response()),
                    resource: defined_type.clone(),
                    warningOnly: Box::new(false.into()),
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
            autocreate: Box::new(false.into()),
            autodelete: Box::new(false.into()),
            resource: Some(Box::new(Reference {
                reference: Some(Box::new(format!("#{}", fixture_id).into())),
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

    testscript.url = Box::new(tag.to_string().into());
    testscript.status = PublicationStatus::active();
    testscript.id = Some(tag.to_string());
    testscript.name = Box::new(tag.to_string().into());

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
                    system: Some(Box::new(
                        "http://terminology.hl7.org/CodeSystem/testscript-operation-codes"
                            .to_string()
                            .into(),
                    )),
                    code: Some(Box::new("deleteCondMultiple".to_string().into())),
                    ..Default::default()
                })),
                encodeRequestUrl: Box::new(true.into()),
                resource: None,
                params: Some(Box::new(format!("_tag={}", tag).into())),
                description: Some(Box::new(
                    "Delete resources created in test.".to_string().into(),
                )),

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
        let walker = WalkDir::new(dir_path).sort_by_file_name().into_iter();
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
