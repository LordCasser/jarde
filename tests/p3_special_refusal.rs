//! Refusal boundaries for non-constructor `invokespecial` calls.
//!
//! The frozen `SpecialProbe` class has stable, uniquely matching bytecode for these adversarial
//! shapes. Each patch asserts the complete instruction bytes and the original pool
//! index before changing only the intended fact.

use jarde::*;
use std::slice;

const SPECIAL: &[u8] = include_bytes!("fixtures/p3-special-dispatch/v8/SpecialProbe.class");

fn unique(bytes: &[u8], pattern: &[u8]) -> usize {
    let matches: Vec<usize> = bytes
        .windows(pattern.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == pattern).then_some(offset))
        .collect();
    assert_eq!(
        matches.len(),
        1,
        "expected one frozen bytecode match for {pattern:02x?}"
    );
    matches[0]
}

fn make_private_helper_public(bytes: &mut [u8]) {
    // method_info header at the frozen privateHelper declaration: private, name #22, descriptor
    // #23, one attribute. Only the access flags change; the target remains Methodref #20.
    let header = [0x00, 0x02, 0x00, 0x16, 0x00, 0x17, 0x00, 0x01];
    let offset = unique(bytes, &header);
    bytes[offset..offset + 2].copy_from_slice(&[0x00, 0x01]);
}

fn patch_call_other_to_base(bytes: &mut [u8]) {
    // callOtherPrivate: aload_1, iload_2, invokespecial #20 privateHelper, ireturn. Replace the
    // target with existing Methodref #27 BaseProbe.valueWith(I)I, retaining the non-this receiver.
    let original = [0x2b, 0x1c, 0xb7, 0x00, 0x14, 0xac];
    let offset = unique(bytes, &original);
    bytes[offset + 3..offset + 5].copy_from_slice(&[0x00, 0x1b]);
}

fn patch_nested_to_nonprivate(bytes: &mut Vec<u8>, discard_result: bool) {
    // superWithSideEffect: aload_0, invokestatic #24, invokespecial #27 BaseProbe.valueWith(I)I,
    // ireturn. After privateHelper becomes public, #20 is a current-class non-private special.
    let original = [0x2a, 0xb8, 0x00, 0x18, 0xb7, 0x00, 0x1b, 0xac];
    let offset = unique(bytes, &original);
    bytes[offset + 5..offset + 7].copy_from_slice(&[0x00, 0x14]);
    if discard_result {
        // Keep the method descriptor ()I while turning the special call into a discarded-result
        // statement: pop, push zero, return. The code and Code attribute lengths both grow by 2.
        bytes.splice(offset + 7..offset + 8, [0x57, 0x03, 0xac]);
        let code_length = u32::from_be_bytes(bytes[offset - 4..offset].try_into().unwrap());
        let attribute_length =
            u32::from_be_bytes(bytes[offset - 12..offset - 8].try_into().unwrap());
        assert_eq!(code_length, 8, "the frozen method has eight code bytes");
        assert_eq!(
            attribute_length, 20,
            "the frozen Code has no nested attributes"
        );
        bytes[offset - 4..offset].copy_from_slice(&(code_length + 2).to_be_bytes());
        bytes[offset - 12..offset - 8].copy_from_slice(&(attribute_length + 2).to_be_bytes());
    }
}

fn budget() -> Budget {
    task_budget(&[]).expect("default task budget is available")
}

fn open(bytes: &[u8]) -> ArtifactSnapshot {
    Engine::new()
        .open(ArtifactInput::bytes(bytes.to_vec()), &mut budget())
        .expect("special refusal fixture opens")
}

fn class_source_of(snapshot: &ArtifactSnapshot) -> ClassSourceReport {
    let request = ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal("SpecialProbe"),
        },
        environment: EnvironmentRequest {
            snapshot: snapshot.id().clone(),
            scope: PhysicalScope::SnapshotAll,
            policy: EnvironmentPolicy::SingleClass,
            profile: RuntimeProfile {
                java_release: 8,
                multi_release: MultiReleasePolicy::Disabled,
                layout: LayoutMode::Generic,
            },
            loader: LoaderId("app".to_owned()),
        },
    };
    match Engine::new()
        .class_source_with_evidence(
            slice::from_ref(snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut budget(),
        )
        .expect("special refusal class-source request succeeds")
    {
        OperationOutcome::Performed(report) => report,
        OperationOutcome::Ambiguous(candidates) => panic!(
            "one standalone fixture must have one candidate, got {}",
            candidates.candidates.len()
        ),
        OperationOutcome::Incomplete(candidates) => panic!(
            "one standalone fixture must complete candidate selection, got {}",
            candidates.candidates.len()
        ),
    }
}

fn recovered<'a>(report: &'a ClassSourceReport, name: &str) -> &'a RecoveryReport {
    let method = report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == name.as_bytes())
        .unwrap_or_else(|| panic!("method `{name}` is absent from SpecialProbe"));
    let ClassSourceOutcome::Recovered { report, .. } = &method.outcome else {
        panic!("method `{name}` was not recovered: {:?}", method.outcome);
    };
    report
}

fn quoted_bcis(text: &str) -> Vec<u32> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("// @bytecode "))
        .flat_map(|list| list.split_whitespace())
        .map(|bci| bci.parse().expect("quoted BCI is numeric"))
        .collect()
}

#[test]
fn nonprivate_current_special_is_refused_instead_of_becoming_this_virtual() {
    let mut bytes = SPECIAL.to_vec();
    make_private_helper_public(&mut bytes);
    let report = class_source_of(&open(&bytes));
    let body = recovered(&report, "callOwnPrivate");
    assert!(
        !quoted_bcis(&body.text).is_empty(),
        "non-private current-class special must remain a quoted refusal:\n{}",
        body.text
    );
    assert!(
        !body.text.contains("privateHelper("),
        "a non-private special must not be silently rewritten as a virtual call:\n{}",
        body.text
    );
    assert!(
        !body.text.contains("this.privateHelper"),
        "a non-private current-class special must not be claimed as private this dispatch:\n{}",
        body.text
    );
}

#[test]
fn nonthis_direct_super_special_is_refused_instead_of_becoming_super() {
    let mut bytes = SPECIAL.to_vec();
    patch_call_other_to_base(&mut bytes);
    let report = class_source_of(&open(&bytes));
    let body = recovered(&report, "callOtherPrivate");
    assert!(
        !quoted_bcis(&body.text).is_empty(),
        "a direct-super target on a non-this receiver must be refused:\n{}",
        body.text
    );
    assert!(
        !body.text.contains("valueWith("),
        "the non-this special must not be emitted as an ordinary receiver call:\n{}",
        body.text
    );
    assert!(
        !body.text.contains("super.valueWith"),
        "the non-this special must not be emitted as super:\n{}",
        body.text
    );
}

#[test]
fn refused_nested_special_keeps_argument_outer_call_and_return_sources() {
    let mut bytes = SPECIAL.to_vec();
    make_private_helper_public(&mut bytes);
    patch_nested_to_nonprivate(&mut bytes, false);
    let report = class_source_of(&open(&bytes));
    let body = recovered(&report, "superWithSideEffect");
    let quoted = quoted_bcis(&body.text);
    for bci in [1, 4, 7] {
        assert!(
            quoted.contains(&bci),
            "refused nested special must quote BCI {bci}; got {quoted:?}:\n{}",
            body.text
        );
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "refused nested special must map BCI {bci}: {:?}",
            body.source_map.segments()
        );
    }
}

#[test]
fn discarded_result_special_keeps_argument_and_call_sources() {
    let mut bytes = SPECIAL.to_vec();
    make_private_helper_public(&mut bytes);
    patch_nested_to_nonprivate(&mut bytes, true);
    let report = class_source_of(&open(&bytes));
    let body = recovered(&report, "superWithSideEffect");
    let quoted = quoted_bcis(&body.text);
    for bci in [1, 4] {
        assert!(
            quoted.contains(&bci),
            "discarded-result special must quote BCI {bci}; got {quoted:?}:\n{}",
            body.text
        );
        assert!(
            !body.source_map.of_bci(bci).is_empty(),
            "discarded-result special must map BCI {bci}: {:?}",
            body.source_map.segments()
        );
    }
}
