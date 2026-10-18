use jarde::*;
use jarde_reader::classfile::{
    ClassFacts, CpEntryKind, MethodCodeFacts, class_facts, cp_class_name, cp_entry,
    method_code_facts,
};
use jarde_reader::ledger::{LedgerEntryTiming, LedgerObserver, LedgerSite};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

const FIXED: &[u8] = include_bytes!("fixtures/cf11-structured-list-api/UpstreamNestedLoops.class");
const DISTINCT_STRING: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-api/DistinctStringFixture.class");
const DISTINCT_INTEGER: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-api/DistinctIntegerFixture.class");
const CONTROLS: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-controls/UpstreamNestedLoops.class");

const INNER_OPTION_INTEGER: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-inner-option/DistinctIntegerFixture.class");

const NULL_STRING: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-null-api/NullStringListFixture.class");
const NULL_INTEGER: &[u8] =
    include_bytes!("fixtures/cf11-structured-list-null-api/NullIntegerListFixture.class");
const ORDINARY_TRY_ADMISSION: &[u8] = include_bytes!(
    "fixtures/cf11-ordinary-try-list-generic/admission/OrdinaryTryListFixture.class"
);

const TYPED_LIST_FOREACH: &[u8] =
    include_bytes!("fixtures/typed-list-foreach/TypedListForEachCases.class");

const NESTED_LOOPS_DEBUG: &[u8] =
    include_bytes!("fixtures/foreach-local-bindings/debug/TestNestedLoops$TestCls.class");
const NESTED_LOOPS_NONE: &[u8] =
    include_bytes!("fixtures/foreach-local-bindings/none/TestNestedLoops$TestCls.class");

fn request(snapshot: &ArtifactSnapshot, name: &str) -> ClassSourceRequest {
    ClassSourceRequest {
        class: ClassRef::Name {
            class: ClassNameQuery::internal(name),
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
    }
}

fn report_with(
    bytes: &[u8],
    name: &str,
    evidence: Option<&RecoveryEvidenceRequest>,
    budget: &mut Budget,
) -> OperationOutcome<ClassSourceReport> {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(bytes.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = request(&snapshot, name);
    match evidence {
        Some(evidence) => engine.class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            evidence,
            budget,
        ),
        None => engine.class_source(std::slice::from_ref(&snapshot), &request, budget),
    }
    .unwrap()
}

#[derive(Debug)]
struct EntryObserver {
    entries: AtomicUsize,
    cancel_after: Option<usize>,
    cancellation: CancellationToken,
}

impl EntryObserver {
    fn new(cancel_after: Option<usize>, cancellation: CancellationToken) -> Self {
        Self {
            entries: AtomicUsize::new(0),
            cancel_after,
            cancellation,
        }
    }

    fn count(&self) -> usize {
        self.entries.load(Ordering::SeqCst)
    }
}

impl LedgerObserver for EntryObserver {
    fn entry(&self, _site: LedgerSite, _timing: Option<LedgerEntryTiming>) {
        let observed = self.entries.fetch_add(1, Ordering::SeqCst) + 1;
        if self.cancel_after == Some(observed) {
            self.cancellation.cancel();
        }
    }

    fn refusal(&self, _site: LedgerSite, _dimension: Option<BudgetDimension>) {}
}

fn observed_admission_report(
    cancellation: CancellationToken,
    observer: Arc<EntryObserver>,
) -> Result<OperationOutcome<ClassSourceReport>> {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(ORDINARY_TRY_ADMISSION.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = request(&snapshot, "cf11/OrdinaryTryListFixture");
    let mut budget = Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation);
    let ledger = OperationLedger::new(&budget).with_observer(observer);
    budget.with_ledger(ledger, UsageOwner::Methods);
    engine.class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
}

fn performed(bytes: &[u8], name: &str) -> ClassSourceReport {
    match report_with(bytes, name, None, &mut task_budget(&[]).unwrap()) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected class-source result: {other:?}"),
    }
}

fn method<'a>(report: &'a ClassSourceReport, method_name: &[u8]) -> &'a ClassSourceMethod {
    report
        .methods
        .iter()
        .find(|method| method.item.name.raw().0 == method_name)
        .unwrap_or_else(|| panic!("missing method {:?}", String::from_utf8_lossy(method_name)))
}

fn raw_body<'a>(method: &'a ClassSourceMethod) -> &'a RecoveryReport {
    match &method.outcome {
        ClassSourceOutcome::Recovered { report, .. } => report,
        other => panic!(
            "method {:?} did not recover: {other:?}",
            method.item.name.raw().0
        ),
    }
}

fn assert_utf8_source_map(report: &RecoveryReport) {
    assert!(!report.source_map.is_empty(), "source map is present");
    for segment in report.source_map.segments() {
        assert!(report.text.is_char_boundary(segment.start()));
        assert!(report.text.is_char_boundary(segment.end()));
        assert_eq!(
            segment.text(&report.text),
            &report.text[segment.start()..segment.end()]
        );
    }
}

fn assert_bcis_are_mapped(report: &RecoveryReport, bcis: &[u32]) {
    for bci in bcis {
        assert!(
            !report.source_map.of_bci(*bci).is_empty(),
            "physical BCI {bci} has source-map provenance"
        );
    }
}

fn assert_folded_typed_foreach(
    projected: &str,
    element_type: &str,
    outer_name: &str,
    outer_source: &str,
    inner_name: &str,
    inner_source: &str,
) {
    for header in [
        format!("for ({element_type} {outer_name} : {outer_source}) {{"),
        format!("for ({element_type} {inner_name} : {inner_source}) {{"),
    ] {
        assert!(
            projected.lines().any(|line| line.trim() == header),
            "folded foreach header {header:?} is retained: {projected}"
        );
    }
    assert!(
        !projected.contains(&format!("{outer_name} = {outer_name};")),
        "outer synthetic-element copy is folded: {projected}"
    );
    assert!(
        !projected.contains(&format!("{element_type} {inner_name} = {inner_name};")),
        "inner synthetic-element copy is folded: {projected}"
    );
    assert!(
        !projected.contains("iteratorElement"),
        "both synthetic names and their body copies are folded: {projected}"
    );
    for removed in [
        format!("{element_type} {outer_name};"),
        format!("{element_type} {inner_name};"),
    ] {
        assert!(
            !projected.lines().any(|line| line.trim() == removed),
            "uninitialized binding declaration is folded: {projected}"
        );
    }
    assert!(
        !projected.contains(&format!("({element_type})")),
        "physical element casts are absent from projected body: {projected}"
    );
}

fn assert_all_instruction_bcis_are_mapped(
    bytes: &[u8],
    class: &ClassFacts,
    member: &ClassSourceMethod,
) {
    let code = method_code(class, bytes, member);
    let bcis = code
        .instructions
        .iter()
        .map(|instruction| instruction.bci)
        .collect::<Vec<_>>();
    assert_bcis_are_mapped(raw_body(member), &bcis);
}

#[test]
fn ordinary_try_admission_helper_preserves_the_field_read_before_its_conditional() {
    let default = performed(ORDINARY_TRY_ADMISSION, "cf11/OrdinaryTryListFixture");
    let all = match report_with(
        ORDINARY_TRY_ADMISSION,
        "cf11/OrdinaryTryListFixture",
        Some(&RecoveryEvidenceRequest::all()),
        &mut task_budget(&[]).unwrap(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("admission class completes: {other:?}"),
    };
    let member = method(&default, b"consume");
    let recovered = raw_body(member);
    assert_eq!(
        recovered.representation,
        Representation::Java,
        "{}",
        member.text
    );
    assert_eq!(recovered.quality, Quality::Structured, "{}", member.text);
    assert_eq!(member.text, method(&all, b"consume").text);
    assert!(!member.text.contains("jarde_refused_body"));
    let read = member
        .text
        .find("int saved0 = cf11.OrdinaryTryListFixture.effects;")
        .expect("the field read is materialized before the conditional");
    let write = member
        .text
        .find("cf11.OrdinaryTryListFixture.effects = saved0 +")
        .expect("the final store uses the saved original value");
    assert!(read < write);
    assert_bcis_are_mapped(recovered, &[0, 3, 4, 7, 11, 12, 13, 16]);
    assert_utf8_source_map(recovered);
}

#[test]
fn forward_conditional_carry_frozen_order_probes_keep_saved_producers_before_the_test() {
    let classes: &[&[u8]] = &[
        include_bytes!("fixtures/forward-conditional-stack-carry/javac8/ForwardCarryProbe.class"),
        include_bytes!("fixtures/forward-conditional-stack-carry/javac23/ForwardCarryProbe.class"),
    ];
    for class in classes {
        let default = performed(class, "cf11/ForwardCarryProbe");
        let all = match report_with(
            class,
            "cf11/ForwardCarryProbe",
            Some(&RecoveryEvidenceRequest::all()),
            &mut task_budget(&[]).unwrap(),
        ) {
            OperationOutcome::Performed(report) => report,
            other => panic!("complete probe class: {other:?}"),
        };
        assert_eq!(default.methods.len(), 16);
        assert_eq!(default.text, all.text);
        for member in &default.methods {
            let raw = raw_body(member);
            assert_eq!(raw.representation, Representation::Java, "{}", member.text);
            assert_eq!(raw.quality, Quality::Structured, "{}", member.text);
            assert_utf8_source_map(raw);
        }
        for name in [
            b"compoundCarry".as_slice(),
            b"compoundRuntimeRhs",
            b"compoundErrorRhs",
        ] {
            let text = &method(&default, name).text;
            let read = text
                .find("int saved0 = cf11.ForwardCarryProbe.effects;")
                .expect("read field once before condition");
            let write = text
                .find("cf11.ForwardCarryProbe.effects = saved0 +")
                .expect("write original field value plus conditional RHS");
            assert!(read < write, "{text}");
        }
        let observed = &method(&default, b"producerPlusObservedTest").text;
        assert_eq!(observed.matches("get()").count(), 1);
        assert!(
            observed.find("int saved0 = get();").unwrap() < observed.find("testFlag(").unwrap()
        );
        let length = &method(&default, b"lengthBeforeConditional").text;
        assert!(length.find(".length;").unwrap() < length.find("lengthArm()").unwrap());
    }
}

#[test]
fn forward_carry_saves_reference_dependencies_before_the_condition() {
    for class in [
        include_bytes!("fixtures/carry-dependency/javac8/CarryDependencyProbe.class").as_slice(),
        include_bytes!("fixtures/carry-dependency/javac23/CarryDependencyProbe.class").as_slice(),
    ] {
        let default = performed(class, "CarryDependencyProbe");
        let all = match report_with(
            class,
            "CarryDependencyProbe",
            Some(&RecoveryEvidenceRequest::all()),
            &mut task_budget(&[]).unwrap(),
        ) {
            OperationOutcome::Performed(report) => report,
            other => panic!("complete dependency probe class: {other:?}"),
        };
        assert_eq!(default.methods.len(), 10);
        assert_eq!(default.text, all.text);
        for member in &default.methods {
            let raw = raw_body(member);
            assert_eq!(raw.representation, Representation::Java, "{}", member.text);
            assert_eq!(raw.quality, Quality::Structured, "{}", member.text);
            assert_utf8_source_map(raw);
            assert_raw_map_method_identity(raw, member);
        }
        for name in [
            b"referenceConsumer".as_slice(),
            b"thisConsumer",
            b"parameterConsumer",
        ] {
            let member = method(&default, name);
            let text = &member.text;
            assert_eq!(text.matches("value()").count(), 1, "{text}");
            assert_eq!(text.matches("test(flag)").count(), 1, "{text}");
            assert_eq!(text.matches("consume(").count(), 1, "{text}");
            assert!(
                text.find("value();").unwrap() < text.find("test(flag)").unwrap(),
                "{text}"
            );
            assert!(!text.contains("could not commit"), "{text}");
            assert!(!text.contains("jarde_refused_body"), "{text}");
        }
        assert_eq!(
            method(&default, b"referenceConsumer")
                .text
                .matches("make()")
                .count(),
            1
        );
    }
}

#[test]
fn failed_carry_receiver_assignment_keeps_the_primary_and_quotes_the_whole_method() {
    for class in [
        include_bytes!(
            "fixtures/carry-dependency/negative/javac8/CarryDependencyRefusalProbe.class"
        )
        .as_slice(),
        include_bytes!(
            "fixtures/carry-dependency/negative/javac23/CarryDependencyRefusalProbe.class"
        )
        .as_slice(),
    ] {
        let default = performed(class, "CarryDependencyRefusalProbe");
        let all = match report_with(
            class,
            "CarryDependencyRefusalProbe",
            Some(&RecoveryEvidenceRequest::all()),
            &mut task_budget(&[]).unwrap(),
        ) {
            OperationOutcome::Performed(report) => report,
            other => panic!("complete refusal control: {other:?}"),
        };
        assert_eq!(default.methods.len(), 2);
        assert_eq!(default.text, all.text);
        let member = method(&default, b"rejectedDependency");
        let raw = raw_body(member);
        assert_eq!(
            raw.content,
            RecoveryContent::ExplanationOnly,
            "{}",
            member.text
        );
        assert!(
            raw.text.contains("saved producer at BCI 7 has 3 consumers"),
            "{}",
            raw.text
        );
        assert!(member.text.contains("jarde_refused_body();"));
        assert!(!member.text.contains("return;"));
        let facts = class_facts(class, &mut task_budget(&[]).unwrap()).unwrap();
        let code = method_code(&facts, class, member);
        let bcis = code
            .instructions
            .iter()
            .map(|instruction| instruction.bci)
            .collect::<Vec<_>>();
        assert_bcis_are_mapped(raw, &bcis);
        assert_utf8_source_map(raw);
        assert_raw_map_method_identity(raw, member);
    }
}

fn assert_raw_map_method_identity(raw: &RecoveryReport, member: &ClassSourceMethod) {
    let bound_method = raw
        .artifact
        .binding()
        .expect("a produced raw body carries its artifact binding")
        .method();
    assert_eq!(
        bound_method, &member.item.identity,
        "raw artifact binding matches the current public method identity"
    );
    for segment in raw.source_map.segments() {
        let origins = std::iter::once(segment.origin().primary()).chain(segment.origin().derived());
        for origin in origins {
            assert_eq!(
                origin.method(),
                Some(bound_method),
                "every mapped anchor names the raw artifact's bound method"
            );
            assert_eq!(
                origin.method(),
                Some(&member.item.identity),
                "every mapped anchor names the current public method identity"
            );
        }
    }
}

fn assert_bci_maps_to(member: &ClassSourceMethod, bci: u32, expected: &str) {
    let report = raw_body(member);
    let mapped = report
        .source_map
        .of_bci(bci)
        .iter()
        .map(|segment| segment.text(&report.text))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        mapped.contains(expected),
        "physical BCI {bci} should map to {expected:?}, got {mapped:?}"
    );
}

fn method_code(class: &ClassFacts, bytes: &[u8], method: &ClassSourceMethod) -> MethodCodeFacts {
    let name = method.item.name.raw().0.as_slice();
    let descriptor = method.item.descriptor.raw().0.as_slice();
    let header = class
        .methods
        .iter()
        .find(|header| header.name.raw().0 == name && header.descriptor.raw().0 == descriptor)
        .expect("public member identity resolves in the same class facts");
    method_code_facts(bytes, header, &mut task_budget(&[]).unwrap())
        .expect("the admission method body decodes completely")
}

fn assert_public_instruction(bytes: &[u8], method: &ClassSourceMethod, bci: u32, opcode: u8) {
    let inspection = inspect_method_bytecode(
        bytes,
        MethodSelector {
            name: method.item.name.raw().clone(),
            descriptor: method.item.descriptor.raw().clone(),
        },
        &mut task_budget(&[]).unwrap(),
    )
    .expect("the public bytecode inspector reads the selected body");
    let instruction = inspection
        .instructions
        .iter()
        .find(|instruction| instruction.bci == bci)
        .expect("the selected physical instruction is present");
    assert_eq!(instruction.opcode, opcode);
}

fn assert_iterator_next_checkcast(
    classfile: &ClassFacts,
    bytes: &[u8],
    method: &ClassSourceMethod,
    next_bci: u32,
    cast_bci: u32,
    element_class: &str,
) {
    let code = method_code(classfile, bytes, method);
    let next = code
        .instructions
        .iter()
        .find(|instruction| instruction.bci == next_bci)
        .expect("the Iterator.next instruction is present");
    assert_eq!(next.opcode, 0xb9, "next call is invokeinterface");
    assert_public_instruction(bytes, method, next_bci, 0xb9);
    let next_index = next
        .constant_pool_index
        .expect("invokeinterface has a CP index");
    assert!(matches!(
        &cp_entry(&classfile.constant_pool, next_index).unwrap().kind,
        CpEntryKind::InterfaceMethodRef { owner, name, descriptor, .. }
            if owner.0 == b"java/util/Iterator"
                && name.0 == b"next"
                && descriptor.0 == b"()Ljava/lang/Object;"
    ));
    let cast = code
        .instructions
        .iter()
        .find(|instruction| instruction.bci == cast_bci)
        .expect("the element checkcast is present");
    assert_eq!(cast.opcode, 0xc0, "element conversion is checkcast");
    assert_public_instruction(bytes, method, cast_bci, 0xc0);
    assert_eq!(
        cast_bci,
        next_bci + next.width,
        "checkcast immediately follows next"
    );
    let cast_index = cast.constant_pool_index.expect("checkcast has a CP index");
    assert_eq!(
        cp_class_name(&classfile.constant_pool, cast_index)
            .unwrap()
            .0,
        element_class.replace('.', "/").as_bytes(),
    );
}

fn assert_remove_object_call(
    classfile: &ClassFacts,
    bytes: &[u8],
    method: &ClassSourceMethod,
    bci: u32,
) {
    let code = method_code(classfile, bytes, method);
    let instruction = code
        .instructions
        .iter()
        .find(|instruction| instruction.bci == bci)
        .expect("the remove call is present");
    assert_eq!(
        instruction.opcode, 0xb9,
        "remove(Object) is invokeinterface"
    );
    assert_public_instruction(bytes, method, bci, 0xb9);
    let index = instruction
        .constant_pool_index
        .expect("invokeinterface has a CP index");
    assert!(matches!(
        &cp_entry(&classfile.constant_pool, index).unwrap().kind,
        CpEntryKind::InterfaceMethodRef { owner, name, descriptor, .. }
            if owner.0 == b"java/util/List"
                && name.0 == b"remove"
                && descriptor.0 == b"(Ljava/lang/Object;)Z"
    ));
}

fn interface_call_bci(
    classfile: &ClassFacts,
    bytes: &[u8],
    method: &ClassSourceMethod,
    expected_owner: &[u8],
    expected_name: &[u8],
    expected_descriptor: &[u8],
) -> u32 {
    let code = method_code(classfile, bytes, method);
    code.instructions
        .iter()
        .find(|instruction| {
            instruction.opcode == 0xb9
                && instruction.constant_pool_index.is_some_and(|index| {
                    matches!(
                        &cp_entry(&classfile.constant_pool, index).unwrap().kind,
                        CpEntryKind::InterfaceMethodRef { owner, name, descriptor, .. }
                            if owner.0 == expected_owner
                                && name.0 == expected_name
                                && descriptor.0 == expected_descriptor
                    )
                })
        })
        .expect("the expected invokeinterface exists in the method body")
        .bci
}

fn assert_generic_refusal(method: &ClassSourceMethod, name: &str, descriptor: &str) {
    let marker = format!(
        "generic Signature projection refused for `{name}{descriptor}`: unsupported (ordinary_generic_source_unproved):"
    );
    assert!(
        method.text.contains(&marker),
        "expected exact generic refusal marker {marker:?}: {}",
        method.text
    );
    assert!(
        matches!(&method.outcome, ClassSourceOutcome::Recovered { .. }),
        "generic refusal must retain the original raw RecoveryReport for {name}: {:?}",
        method.outcome
    );
    let raw = raw_body(method);
    assert_utf8_source_map(raw);
    assert!(
        !raw.text.is_empty(),
        "the original raw RecoveryReport is retained"
    );
}

fn assert_admission_projection_absent(report: &ClassSourceReport) {
    for (name, declaration) in [
        (
            "renamedTry",
            "renamedTry(java.util.List<java.lang.String> source, java.util.List<java.lang.String> sink)",
        ),
        (
            "stringBranch",
            "stringBranch(java.util.List<java.lang.String> input, java.util.List<java.lang.String> output)",
        ),
        (
            "integerTry",
            "integerTry(java.util.List<java.lang.Integer> selected, java.util.List<java.lang.Integer> available)",
        ),
        (
            "nestedTry",
            "nestedTry(java.util.List<java.lang.String> source, java.util.List<java.lang.String> witness, java.util.List<java.lang.String> destination)",
        ),
        (
            "stringRemovePrefix",
            "stringRemovePrefix(java.util.List<java.lang.String> input, java.util.List<java.lang.String> output, java.lang.Object key)",
        ),
        (
            "catchVisibleIteratorGap",
            "catchVisibleIteratorGap(java.util.List<java.lang.String> input, java.util.List<java.lang.String> output)",
        ),
    ] {
        assert!(
            !report.text.contains(declaration),
            "a stopped class cannot publish staged generic declaration {declaration:?}"
        );
        let member = method(report, name.as_bytes());
        assert!(
            !member.text.contains(declaration)
                && !member
                    .declaration
                    .as_deref()
                    .unwrap_or_default()
                    .contains("<java.lang."),
            "a stopped method cannot publish generic sidecars for {name}: {}",
            member.text
        );
    }
    assert!(
        method(report, b"renamedTry")
            .text
            .contains("sink.add((java.lang.Object) element)"),
        "the raw add(E) body remains intact until the generic overlay commits: {}",
        method(report, b"renamedTry").text
    );
}

#[test]
fn nearby_list_controls_refuse_only_unproved_generic_projections() {
    let report = performed(CONTROLS, "cf11/UpstreamNestedLoops");
    let controls_classfile = class_facts(CONTROLS, &mut task_budget(&[]).unwrap())
        .expect("the nearby-controls class facts and constant pool decode");

    // These are distinct source-proof failures around the admitted CF11 body. Check only that
    // the generic layer refuses and the original raw RecoveryReport plus map remain available;
    // this does not claim the recovered presentation is a compilable complete source file.
    for (name, descriptor) in [
        ("rawAliasEscape", "(Ljava/util/List;Ljava/util/List;)V"),
        ("reassigned", "(Ljava/util/List;Ljava/util/List;)V"),
        ("wildcardNull", "(Ljava/util/List;Ljava/util/List;)V"),
        (
            "indexOverload",
            "(Ljava/util/List;Ljava/util/List;Ljava/lang/Integer;I)V",
        ),
    ] {
        assert_generic_refusal(method(&report, name.as_bytes()), name, descriptor);
    }

    // A raw-signature method is a raw path, not evidence of a generic refusal.
    let raw = method(&report, b"rawLists");
    assert!(
        !raw.text.contains("generic Signature projection refused"),
        "raw parameters remain on their existing raw path: {}",
        raw.text
    );

    // Keep the physical cast boundary independent of the generic refusal controls. Its raw
    // recovery artifact must still mention the real String conversions and source map.
    let casts = method(&report, b"castBoundary");
    let raw_casts = raw_body(casts);
    assert!(raw_casts.text.contains("java.lang.String") || raw_casts.text.contains("String"));
    assert_utf8_source_map(raw_casts);
    assert_bcis_are_mapped(raw_casts, &[26, 60]);

    // A RuntimeException handler with a directly used List still projects precisely. The raw
    // body and its physical String cast remain available with method-bound source-map anchors.
    let handled = method(&report, b"handled");
    assert!(
        handled.declaration.as_deref().is_some_and(|declaration| {
            declaration.ends_with(
                "handled(java.util.List<java.lang.String> input, java.util.List<java.lang.String> output)",
            )
        }),
        "handler-bearing direct List use gets the exact List<String> declaration: {}",
        handled.text
    );
    let handled_raw = raw_body(handled);
    assert!(
        handled_raw
            .text
            .contains("java.lang.String value = (java.lang.String)"),
        "{}",
        handled_raw.text
    );
    assert!(
        handled_raw.text.contains("iteratorElement"),
        "{}",
        handled_raw.text
    );
    assert_utf8_source_map(handled_raw);
    assert_raw_map_method_identity(handled_raw, handled);
    let next_bci = interface_call_bci(
        &controls_classfile,
        CONTROLS,
        handled,
        b"java/util/Iterator",
        b"next",
        b"()Ljava/lang/Object;",
    );
    let code = method_code(&controls_classfile, CONTROLS, handled);
    let next = code
        .instructions
        .iter()
        .find(|instruction| instruction.bci == next_bci)
        .unwrap();
    let cast_bci = next_bci + next.width;
    assert_iterator_next_checkcast(
        &controls_classfile,
        CONTROLS,
        handled,
        next_bci,
        cast_bci,
        "java.lang.String",
    );
    assert_bci_maps_to(handled, next_bci, "iteratorElement");
    assert_bci_maps_to(handled, cast_bci, "java.lang.String");
}

#[test]
fn ordinary_try_list_projection_covers_exact_types_catch_uses_and_topology_controls() {
    let admission_classfile = class_facts(ORDINARY_TRY_ADMISSION, &mut task_budget(&[]).unwrap())
        .expect("the admission fixture class facts and constant pool decode");
    let default = performed(ORDINARY_TRY_ADMISSION, "cf11/OrdinaryTryListFixture");
    let all = match report_with(
        ORDINARY_TRY_ADMISSION,
        "cf11/OrdinaryTryListFixture",
        Some(&RecoveryEvidenceRequest::all()),
        &mut task_budget(&[]).unwrap(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected all-evidence result: {other:?}"),
    };

    assert_eq!(default.text, all.text);
    assert_eq!(default.methods.len(), all.methods.len());
    for (default_method, all_method) in default.methods.iter().zip(&all.methods) {
        assert_eq!(
            default_method.item.name.raw().0,
            all_method.item.name.raw().0
        );
        assert_eq!(default_method.text, all_method.text);
    }

    // `renamedTry` changes only source names from a proven branch topology. `stringBranch`
    // separately exercises the selected direct catch-owned List use.
    for (name, descriptor, declaration, source_anchor) in [
        (
            "renamedTry",
            "(Ljava/util/List;Ljava/util/List;)V",
            "renamedTry(java.util.List<java.lang.String> source, java.util.List<java.lang.String> sink)",
            "sink.add(element)",
        ),
        (
            "stringBranch",
            "(Ljava/util/List;Ljava/util/List;)V",
            "stringBranch(java.util.List<java.lang.String> input, java.util.List<java.lang.String> output)",
            "output.add(null)",
        ),
        (
            "integerTry",
            "(Ljava/util/List;Ljava/util/List;)V",
            "integerTry(java.util.List<java.lang.Integer> selected, java.util.List<java.lang.Integer> available)",
            "available.add(value)",
        ),
        (
            "nestedTry",
            "(Ljava/util/List;Ljava/util/List;Ljava/util/List;)V",
            "nestedTry(java.util.List<java.lang.String> source, java.util.List<java.lang.String> witness, java.util.List<java.lang.String> destination)",
            "destination.add(outer)",
        ),
        (
            "stringRemovePrefix",
            "(Ljava/util/List;Ljava/util/List;Ljava/lang/Object;)V",
            "stringRemovePrefix(java.util.List<java.lang.String> input, java.util.List<java.lang.String> output, java.lang.Object key)",
            "output.remove(key)",
        ),
    ] {
        let member = method(&default, name.as_bytes());
        assert_eq!(member.item.descriptor.raw().0, descriptor.as_bytes());
        assert!(
            member
                .declaration
                .as_deref()
                .is_some_and(|actual| actual.ends_with(declaration)),
            "exact projected declaration should end in {declaration:?}: {}",
            member.text
        );
        assert!(member.text.contains(source_anchor), "{}", member.text);
        assert!(
            !member.text.contains(".add((java.lang.Object)"),
            "proved add(E) keeps its source-level element type: {}",
            member.text
        );
        assert!(matches!(
            &member.outcome,
            ClassSourceOutcome::Recovered { .. }
        ));
        let default_raw = raw_body(member);
        let all_raw = raw_body(method(&all, name.as_bytes()));
        assert_eq!(default_raw.text, all_raw.text);
        assert_eq!(default_raw.source_map, all_raw.source_map);
        let element_class = if name == "integerTry" {
            "java.lang.Integer"
        } else {
            "java.lang.String"
        };
        assert!(
            default_raw.text.contains(element_class)
                && default_raw.text.contains("iteratorElement"),
            "the physical Iterator.next checkcast for {element_class} remains in raw recovery: {}",
            default_raw.text
        );
        assert_utf8_source_map(default_raw);
        assert_raw_map_method_identity(default_raw, member);
        if name == "renamedTry" || name == "stringBranch" || name == "integerTry" {
            // BCI 17/22 are checked against this fixture's actual Code and constant pool below:
            // invokeinterface Iterator.next()Ljava/lang/Object; then checkcast to List<E>.
            assert_iterator_next_checkcast(
                &admission_classfile,
                ORDINARY_TRY_ADMISSION,
                member,
                17,
                22,
                element_class,
            );
            assert_bci_maps_to(member, 17, "iteratorElement");
            assert_bci_maps_to(member, 22, element_class);
        }
        if name == "nestedTry" {
            // Two independently checked Code/constant-pool pairs, one per foreach iterator.
            for (next_bci, cast_bci) in [(20, 25), (50, 55)] {
                assert_iterator_next_checkcast(
                    &admission_classfile,
                    ORDINARY_TRY_ADMISSION,
                    member,
                    next_bci,
                    cast_bci,
                    element_class,
                );
                assert_bci_maps_to(member, next_bci, "iteratorElement");
                assert_bci_maps_to(member, cast_bci, element_class);
            }
        }
        if name == "stringRemovePrefix" {
            // Assert the actual List.remove(Object) member ref and the following foreach's
            // Iterator.next/checkcast pair. The Object formal means no source cast is needed.
            assert_remove_object_call(&admission_classfile, ORDINARY_TRY_ADMISSION, member, 2);
            assert_iterator_next_checkcast(
                &admission_classfile,
                ORDINARY_TRY_ADMISSION,
                member,
                28,
                33,
                element_class,
            );
            assert_bci_maps_to(member, 2, "remove(");
            assert_bci_maps_to(member, 2, "key");
            assert_bci_maps_to(member, 28, "iteratorElement");
            assert_bci_maps_to(member, 33, element_class);
        }
    }
    assert!(
        method(&default, b"stringBranch")
            .text
            .contains("output.add(value)"),
        "String branch retains non-null element insertion"
    );
    assert!(
        method(&default, b"integerTry")
            .text
            .contains("available.add(null)"),
        "Integer branch retains null insertion"
    );
    let catch_body = method(&default, b"stringBranch")
        .text
        .split("catch (java.lang.RuntimeException failure)")
        .nth(1)
        .unwrap_or("");
    assert!(
        catch_body.contains("output.add(null)"),
        "direct catch-owned List use remains in the body: {}",
        method(&default, b"stringBranch").text
    );
    let remove = method(&default, b"stringRemovePrefix");
    assert!(
        remove.text.contains("output.remove(key)"),
        "{}",
        remove.text
    );
    assert!(
        !remove.text.contains("output.remove((int)"),
        "remove(int) was selected: {}",
        remove.text
    );
    let nested = method(&default, b"nestedTry");
    assert!(nested.text.contains("inner"), "{}", nested.text);

    // These methods have structured ordinary-try bodies, but one part of the whole-method
    // use/topology proof is deliberately outside the finite generic List contract.
    for name in [
        "unknownCatchUse",
        "reassignedInCatch",
        "rawAliasInCatch",
        "multiCatch",
        "catchVisibleIteratorGap",
    ] {
        assert_generic_refusal(
            method(&default, name.as_bytes()),
            name,
            "(Ljava/util/List;Ljava/util/List;)V",
        );
    }
    let gap_method = method(&default, b"catchVisibleIteratorGap");
    let gap_raw = raw_body(gap_method);
    assert_raw_map_method_identity(gap_raw, gap_method);
    assert!(gap_raw.text.contains("marker"), "{}", gap_raw.text);
    assert!(gap_raw.text.contains("effects"), "{}", gap_raw.text);
    let iterator_bci = interface_call_bci(
        &admission_classfile,
        ORDINARY_TRY_ADMISSION,
        gap_method,
        b"java/util/List",
        b"iterator",
        b"()Ljava/util/Iterator;",
    );
    assert_bci_maps_to(gap_method, iterator_bci, "iterator()");
    assert_generic_refusal(
        method(&default, b"indexOverload"),
        "indexOverload",
        "(Ljava/util/List;Ljava/util/List;Ljava/lang/Integer;I)V",
    );
}

#[test]
fn inner_list_option_actual_uses_the_receiver_element_type() {
    let report = performed(INNER_OPTION_INTEGER, "cf11/DistinctIntegerFixture");
    let integer_method = method(&report, b"numbers");
    let integer_raw = raw_body(integer_method);
    let declaration = "numbers(java.util.List<java.lang.Integer> selected, java.util.List<java.lang.Integer> available)";
    assert!(report.text.contains(declaration), "{}", report.text);
    assert!(
        integer_method.text.contains(declaration),
        "{}",
        integer_method.text
    );
    assert!(
        integer_method.text.contains("available.add(option)"),
        "{}",
        integer_method.text
    );
    assert!(
        !integer_method.text.contains(".add((java.lang.Object)"),
        "synthetic add(E) wrapper remains: {}",
        integer_method.text
    );
    assert!(
        integer_method.text.contains("selected.remove(option)"),
        "proved reference argument still carries a redundant Object wrapper: {}",
        integer_method.text
    );
    assert!(
        !integer_method.text.contains("selected.remove((int)"),
        "remove(int) was selected: {}",
        integer_method.text
    );
    assert_utf8_source_map(integer_raw);
    assert_bcis_are_mapped(integer_raw, &[22, 52, 76, 77, 79, 96, 97, 99, 128, 129]);
    for cast_bci in [22, 52] {
        assert!(
            integer_raw
                .source_map
                .of_bci(cast_bci)
                .iter()
                .any(|segment| {
                    segment
                        .text(&integer_raw.text)
                        .contains("java.lang.Integer")
                        || segment.text(&integer_raw.text).contains("Integer")
                }),
            "physical Integer checkcast at BCI {cast_bci} remains in raw recovery source"
        );
    }
}

#[test]
fn fixed_nested_string_body_projects_source_and_preserves_physical_origins() {
    let default = performed(FIXED, "cf11/UpstreamNestedLoops");
    let all = match report_with(
        FIXED,
        "cf11/UpstreamNestedLoops",
        Some(&RecoveryEvidenceRequest::all()),
        &mut task_budget(&[]).unwrap(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected all-evidence result: {other:?}"),
    };
    let no_rule_details_request =
        RecoveryEvidenceRequest::essential().with_kind(RecoveryEvidenceKind::SourceMap);
    let no_rule_details = match report_with(
        FIXED,
        "cf11/UpstreamNestedLoops",
        Some(&no_rule_details_request),
        &mut task_budget(&[]).unwrap(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected no-RuleDetails result: {other:?}"),
    };

    assert_eq!(default.text, all.text);
    assert_eq!(default.text, no_rule_details.text);
    let default_method = method(&default, b"test");
    let all_method = method(&all, b"test");
    let no_rule_details_method = method(&no_rule_details, b"test");
    let default_raw = raw_body(default_method);
    let all_raw = raw_body(all_method);
    let no_rule_details_raw = raw_body(no_rule_details_method);
    assert_eq!(default_method.text, all_method.text);
    assert_eq!(default_method.text, no_rule_details_method.text);
    assert_eq!(default_raw.text, all_raw.text);
    assert_eq!(default_raw.text, no_rule_details_raw.text);
    assert_eq!(default_raw.source_map, all_raw.source_map);
    assert_eq!(default_raw.source_map, no_rule_details_raw.source_map);

    for projected in [
        &default.text,
        &default_method.text,
        &all_method.text,
        &no_rule_details_method.text,
    ] {
        assert_folded_typed_foreach(projected, "java.lang.String", "s1", "l1", "s2", "l2");
    }

    let declaration =
        "test(java.util.List<java.lang.String> l1, java.util.List<java.lang.String> l2)";
    assert!(
        default.text.contains(declaration),
        "full List<String> class declaration: {}",
        default.text
    );
    assert!(
        default_method.text.contains(declaration),
        "full List<String> method text: {}",
        default_method.text
    );
    assert!(
        default_method.text.contains("s1.equals(s2)"),
        "proved String equals argument should not retain the Object wrapper: {}",
        default_method.text
    );
    assert!(
        !default_method
            .text
            .contains("s1.equals((java.lang.Object) s2)"),
        "the synthetic equals wrapper remains: {}",
        default_method.text
    );
    assert!(
        default_method.text.contains("l2.add(s1)"),
        "{}",
        default_method.text
    );
    assert!(
        !default_method.text.contains(".add((java.lang.Object)"),
        "add(E) must not retain the synthetic Object argument wrapper: {}",
        default_method.text
    );
    assert!(
        default_method.text.contains("l1.remove(s2)"),
        "proved String reference should omit the redundant Object wrapper: {}",
        default_method.text
    );
    assert!(
        !default_method.text.contains("l1.remove((int)"),
        "{}",
        default_method.text
    );

    // The source map is defined over the unprojected RecoveryReport::text. It proves physical
    // provenance; it is not a map for ClassSourceMethod::text or ClassSourceReport::text.
    assert_utf8_source_map(default_raw);
    let fixed_class = class_facts(FIXED, &mut task_budget(&[]).unwrap()).unwrap();
    assert_all_instruction_bcis_are_mapped(FIXED, &fixed_class, default_method);
    assert!(
        default_raw.text.contains("for (java.lang.Object "),
        "{}",
        default_raw.text
    );
    assert!(
        default_raw.text.contains("(java.lang.String)"),
        "{}",
        default_raw.text
    );
    assert_bcis_are_mapped(
        default_raw,
        &[
            0, 1, 22, 27, 28, 52, 76, 77, 79, 88, 89, 91, 103, 104, 112, 113,
        ],
    );
    for cast_bci in [22, 52] {
        assert!(
            default_raw
                .source_map
                .of_bci(cast_bci)
                .iter()
                .any(|segment| {
                    segment.text(&default_raw.text).contains("java.lang.String")
                        || segment.text(&default_raw.text).contains("String")
                }),
            "real String checkcast at BCI {cast_bci} remains tied to its raw recovery source"
        );
    }
}

#[test]
fn differently_named_string_and_integer_classes_follow_their_list_element_type() {
    let string = performed(DISTINCT_STRING, "cf11/DistinctStringFixture");
    let string_method = method(&string, b"renamed");
    let string_raw = raw_body(string_method);
    let string_declaration = "renamed(java.util.List<java.lang.String> chosen, java.util.List<java.lang.String> offered)";
    assert!(string.text.contains(string_declaration), "{}", string.text);
    assert!(
        string_method.text.contains(string_declaration),
        "{}",
        string_method.text
    );
    assert!(
        string_method.text.contains("offered.add(candidate)"),
        "{}",
        string_method.text
    );
    assert!(
        !string_method.text.contains(".add((java.lang.Object)"),
        "{}",
        string_method.text
    );
    assert!(
        string_method.text.contains("chosen.remove(option)"),
        "proved reference argument still carries a redundant Object wrapper: {}",
        string_method.text
    );
    for projected in [&string.text, &string_method.text] {
        assert_folded_typed_foreach(
            projected,
            "java.lang.String",
            "candidate",
            "chosen",
            "option",
            "offered",
        );
    }
    assert_utf8_source_map(string_raw);
    let string_class = class_facts(DISTINCT_STRING, &mut task_budget(&[]).unwrap()).unwrap();
    assert_all_instruction_bcis_are_mapped(DISTINCT_STRING, &string_class, string_method);
    assert!(
        string_raw.text.contains("for (java.lang.Object "),
        "{}",
        string_raw.text
    );
    assert!(
        string_raw.text.contains("(java.lang.String)"),
        "{}",
        string_raw.text
    );
    for cast_bci in [22, 52] {
        assert!(
            string_raw
                .source_map
                .of_bci(cast_bci)
                .iter()
                .any(|segment| { segment.text(&string_raw.text).contains("java.lang.String") }),
            "physical String checkcast at BCI {cast_bci} remains in raw recovery source"
        );
    }

    let integer = performed(DISTINCT_INTEGER, "cf11/DistinctIntegerFixture");
    let integer_method = method(&integer, b"numbers");
    let integer_raw = raw_body(integer_method);
    let integer_declaration = "numbers(java.util.List<java.lang.Integer> selected, java.util.List<java.lang.Integer> available)";
    assert!(
        integer.text.contains(integer_declaration),
        "{}",
        integer.text
    );
    assert!(
        integer_method.text.contains(integer_declaration),
        "{}",
        integer_method.text
    );
    assert!(
        integer_method.text.contains("available.add(value)"),
        "{}",
        integer_method.text
    );
    assert!(
        !integer_method.text.contains(".add((java.lang.Object)"),
        "{}",
        integer_method.text
    );
    assert!(
        integer_method.text.contains("selected.remove(option)"),
        "proved reference argument still carries a redundant Object wrapper: {}",
        integer_method.text
    );
    for projected in [&integer.text, &integer_method.text] {
        assert_folded_typed_foreach(
            projected,
            "java.lang.Integer",
            "value",
            "selected",
            "option",
            "available",
        );
    }
    assert_utf8_source_map(integer_raw);
    let integer_class = class_facts(DISTINCT_INTEGER, &mut task_budget(&[]).unwrap()).unwrap();
    assert_all_instruction_bcis_are_mapped(DISTINCT_INTEGER, &integer_class, integer_method);
    assert!(
        integer_raw.text.contains("for (java.lang.Object "),
        "{}",
        integer_raw.text
    );
    assert!(
        integer_raw.text.contains("(java.lang.Integer)"),
        "{}",
        integer_raw.text
    );
    for cast_bci in [22, 52] {
        assert!(
            integer_raw
                .source_map
                .of_bci(cast_bci)
                .iter()
                .any(|segment| {
                    segment
                        .text(&integer_raw.text)
                        .contains("java.lang.Integer")
                }),
            "physical Integer checkcast at BCI {cast_bci} remains in raw recovery source"
        );
    }
}

#[test]
fn exact_list_null_add_removes_only_the_synthetic_object_wrapper() {
    let string = performed(NULL_STRING, "cf11/NullStringListFixture");
    let string_method = method(&string, b"strings");
    let string_raw = raw_body(string_method);
    let string_declaration = "strings(java.util.List<java.lang.String> selected, java.util.List<java.lang.String> available)";
    assert!(string.text.contains(string_declaration), "{}", string.text);
    assert!(
        string_method.text.contains("available.add(null)"),
        "{}",
        string_method.text
    );
    assert!(
        !string_method
            .text
            .contains("available.add((java.lang.Object) null)"),
        "synthetic add(E) wrapper remains: {}",
        string_method.text
    );
    assert!(
        string_method.text.contains("selected.remove(option)"),
        "proved reference argument still carries a redundant Object wrapper: {}",
        string_method.text
    );
    assert_utf8_source_map(string_raw);

    let integer = performed(NULL_INTEGER, "cf11/NullIntegerListFixture");
    let integer_method = method(&integer, b"integers");
    let integer_raw = raw_body(integer_method);
    let integer_declaration = "integers(java.util.List<java.lang.Integer> selected, java.util.List<java.lang.Integer> available)";
    assert!(
        integer.text.contains(integer_declaration),
        "{}",
        integer.text
    );
    assert!(
        integer_method.text.contains("available.add(null)"),
        "{}",
        integer_method.text
    );
    assert!(
        !integer_method
            .text
            .contains("available.add((java.lang.Object) null)"),
        "synthetic add(E) wrapper remains: {}",
        integer_method.text
    );
    assert!(
        integer_method.text.contains("selected.remove(option)"),
        "proved reference argument still carries a redundant Object wrapper: {}",
        integer_method.text
    );
    assert_utf8_source_map(integer_raw);
}

#[test]
fn pre_cancel_does_not_install_a_partial_generic_declaration_or_method_projection() {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(FIXED.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = request(&snapshot, "cf11/UpstreamNestedLoops");
    let token = CancellationToken::new();
    token.cancel();
    let cancelled = engine
        .class_source(
            std::slice::from_ref(&snapshot),
            &request,
            &mut Budget::with_cancellation_token(task_limits(&[]).unwrap(), token),
        )
        .unwrap();
    match cancelled {
        OperationOutcome::Incomplete(selection) => assert!(matches!(
            selection.execution,
            ExecutionReport::Cancelled { .. }
        )),
        OperationOutcome::Performed(report) => {
            assert!(matches!(
                &report.execution,
                ExecutionReport::Cancelled { .. }
            ));
            let generic_declaration =
                "test(java.util.List<java.lang.String> l1, java.util.List<java.lang.String> l2)";
            assert!(
                !report.text.contains(generic_declaration),
                "{}",
                report.text
            );
            if let Some(method) = report
                .methods
                .iter()
                .find(|method| method.item.name.raw().0 == b"test")
            {
                assert!(
                    !method.text.contains(generic_declaration),
                    "{}",
                    method.text
                );
                // A completed raw RecoveryReport may be retained with its original source map.
                // Cancellation only forbids installing a partial generic source projection.
                if let ClassSourceOutcome::Recovered { report, .. } = &method.outcome {
                    assert_utf8_source_map(report);
                }
            }
        }
        OperationOutcome::Ambiguous(_) => panic!("frozen class must bind uniquely"),
    }
}

#[test]
fn ordinary_try_late_cancellation_keeps_raw_source_and_installs_no_partial_projection() {
    let complete_token = CancellationToken::new();
    let complete_observer = Arc::new(EntryObserver::new(None, complete_token.clone()));
    let complete = match observed_admission_report(complete_token, complete_observer.clone()) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("observed admission run must complete: {other:?}"),
    };
    assert!(matches!(
        &complete.execution,
        ExecutionReport::Complete { .. }
    ));
    let target_name = b"renamedTry";
    let complete_method = method(&complete, target_name);
    assert!(complete_method.text.contains("for (java.lang.String "));
    let complete_raw = raw_body(complete_method);
    let projected_declaration = "renamedTry(java.util.List<java.lang.String> source, java.util.List<java.lang.String> sink)";
    assert!(complete_method.text.contains(projected_declaration));

    // Locate the first ledger boundary that commits this target method, rather than cancelling
    // near the whole class's end, after earlier method projections have already committed.
    let mut lower = 0;
    let mut upper = complete_observer.count();
    assert!(upper > 64);
    while lower + 1 < upper {
        let boundary = lower + (upper - lower) / 2;
        let token = CancellationToken::new();
        let observer = Arc::new(EntryObserver::new(Some(boundary), token.clone()));
        let committed = match observed_admission_report(token, observer) {
            Ok(OperationOutcome::Performed(report)) => report.text.contains(projected_declaration),
            Ok(_) | Err(Error::Cancelled { .. }) => false,
            Err(error) => panic!("unexpected search failure: {error:?}"),
        };
        if committed {
            upper = boundary;
        } else {
            lower = boundary;
        }
    }
    let token = CancellationToken::new();
    let observer = Arc::new(EntryObserver::new(Some(lower), token.clone()));
    let report = match observed_admission_report(token, observer) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("the observed projection boundary must retain its class report: {other:?}"),
    };
    assert!(matches!(
        &report.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_admission_projection_absent(&report);
    assert!(
        !method(&report, b"renamedTry")
            .text
            .contains("for (java.lang.String ")
    );
    let raw = raw_body(method(&report, target_name));
    assert_eq!(raw.text, complete_raw.text);
    assert_eq!(raw.source_map, complete_raw.source_map);
    assert_utf8_source_map(raw);
}

#[test]
fn ordinary_try_late_budget_stop_keeps_raw_source_and_installs_no_partial_projection() {
    let complete = performed(ORDINARY_TRY_ADMISSION, "cf11/OrdinaryTryListFixture");
    let complete_method = method(&complete, b"renamedTry");
    let projected_declaration = "renamedTry(java.util.List<java.lang.String> source, java.util.List<java.lang.String> sink)";
    assert!(
        complete_method.text.contains(projected_declaration),
        "the budget-stop check requires a completed ordinary-try List<String> projection: {}",
        complete_method.text
    );
    assert!(complete_method.text.contains("for (java.lang.String "));
    let complete_raw = raw_body(complete_method);
    // A bounded search observes this target's first successful overlay commit. The immediately
    // preceding budget must retain the completed raw artifact and publish no staged projection.
    let mut lower = 0;
    let mut upper = complete.usage.analysis_steps;
    while lower + 1 < upper {
        let boundary = lower + (upper - lower) / 2;
        let mut limits = task_limits(&[]).unwrap();
        limits.analysis_steps = boundary;
        let committed = match report_with(
            ORDINARY_TRY_ADMISSION,
            "cf11/OrdinaryTryListFixture",
            None,
            &mut Budget::new(limits),
        ) {
            OperationOutcome::Performed(report) => report.text.contains(projected_declaration),
            _ => false,
        };
        if committed {
            upper = boundary;
        } else {
            lower = boundary;
        }
    }
    let mut limits = task_limits(&[]).unwrap();
    limits.analysis_steps = lower;
    let report = match report_with(
        ORDINARY_TRY_ADMISSION,
        "cf11/OrdinaryTryListFixture",
        None,
        &mut Budget::new(limits),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("the observed projection boundary must retain its class report: {other:?}"),
    };
    assert!(matches!(
        &report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
            },
            ..
        }
    ));
    assert_admission_projection_absent(&report);
    assert!(
        !method(&report, b"renamedTry")
            .text
            .contains("for (java.lang.String ")
    );
    let raw = raw_body(method(&report, b"renamedTry"));
    assert_eq!(raw.text, complete_raw.text);
    assert_eq!(raw.source_map, complete_raw.source_map);
    assert_utf8_source_map(raw);
}

#[test]
fn budget_stop_during_list_proof_keeps_raw_body_and_installs_no_projection() {
    let complete = performed(FIXED, "cf11/UpstreamNestedLoops");
    let complete_method = method(&complete, b"test");
    let complete_raw = raw_body(complete_method);

    // Keep the original 32_531-step cap. New projection work changes the preceding charge
    // count; the contract is a real refusal under that same cap with the completed raw artifact
    // intact and no staged declaration or typed body published.
    let mut limits = task_limits(&[]).unwrap();
    limits.analysis_steps = 32_531;
    let mut budget = Budget::new(limits);
    let partial = match report_with(FIXED, "cf11/UpstreamNestedLoops", None, &mut budget) {
        OperationOutcome::Performed(report) => report,
        other => panic!("proof-stage budget stop must keep its class report: {other:?}"),
    };
    assert!(matches!(
        &partial.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
            },
            ..
        }
    ));
    assert!(partial.usage.analysis_steps < 32_531);

    let partial_method = method(&partial, b"test");
    assert!(!partial_method.text.contains("for (java.lang.String "));
    let partial_raw = raw_body(partial_method);
    assert_eq!(partial_raw.text, complete_raw.text);
    assert_eq!(partial_raw.source_map, complete_raw.source_map);
    assert_utf8_source_map(partial_raw);

    let generic_declaration =
        "test(java.util.List<java.lang.String> l1, java.util.List<java.lang.String> l2)";
    assert!(
        !partial.text.contains(generic_declaration),
        "{}",
        partial.text
    );
    assert!(
        !partial_method.text.contains(generic_declaration),
        "{}",
        partial_method.text
    );
    assert!(
        partial_method
            .text
            .contains("l2.add((java.lang.Object) s1)"),
        "the raw method presentation keeps its original add argument cast: {}",
        partial_method.text
    );
}

/// Root-only probe for locating the exact newly added projection stop after the production change.
/// Its output is diagnostic; it deliberately makes no claim that usage - 1 reaches that phase.
#[test]
#[ignore = "root diagnostic: inspect observed stop phase and method outcome before pinning lateStop"]
fn diagnose_late_analysis_stop_phase_without_assuming_raw_map_is_empty() {
    let complete = performed(FIXED, "cf11/UpstreamNestedLoops");
    eprintln!("complete analysis_steps={}", complete.usage.analysis_steps);
    for distance in [1_u64, 2, 4, 8, 16, 32, 64] {
        if complete.usage.analysis_steps <= distance {
            continue;
        }
        let limit = complete.usage.analysis_steps - distance;
        let mut limits = task_limits(&[]).unwrap();
        limits.analysis_steps = limit;
        let outcome = report_with(
            FIXED,
            "cf11/UpstreamNestedLoops",
            None,
            &mut Budget::new(limits),
        );
        match outcome {
            OperationOutcome::Performed(report) => {
                eprintln!(
                    "limit={limit} usage={} execution={:?}",
                    report.usage.analysis_steps, report.execution
                );
                eprintln!("class text={}", report.text);
                if let Some(method) = report
                    .methods
                    .iter()
                    .find(|method| method.item.name.raw().0 == b"test")
                {
                    eprintln!(
                        "method declaration={:?} text={}",
                        method.declaration, method.text
                    );
                    match &method.outcome {
                        ClassSourceOutcome::Recovered { report, .. } => eprintln!(
                            "raw outcome=recovered raw_text={} raw_source_map_bcis_79={:?}",
                            report.text,
                            report
                                .source_map
                                .of_bci(79)
                                .iter()
                                .map(|segment| segment.text(&report.text))
                                .collect::<Vec<_>>()
                        ),
                        other => eprintln!("raw/member outcome={other:?}"),
                    }
                }
            }
            other => eprintln!("limit={limit} outcome={other:?}"),
        }
    }
}

#[test]
fn raw_alias_escape_reference_lifetimes_match_default_all_and_source_origins() {
    let default = performed(CONTROLS, "cf11/UpstreamNestedLoops");
    let all = match report_with(
        CONTROLS,
        "cf11/UpstreamNestedLoops",
        Some(&RecoveryEvidenceRequest::all()),
        &mut task_budget(&[]).unwrap(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("unexpected all-evidence result: {other:?}"),
    };

    assert!(matches!(
        &default.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(matches!(&all.execution, ExecutionReport::Complete { .. }));
    assert_eq!(
        default.methods.len(),
        11,
        "the whole class keeps all 11 methods"
    );
    assert_eq!(default.methods.len(), all.methods.len());
    assert_eq!(
        default.text, all.text,
        "default and all preserve the full class text"
    );
    for (default_method, all_method) in default.methods.iter().zip(&all.methods) {
        assert_eq!(
            default_method.item.name.raw().0,
            all_method.item.name.raw().0,
            "method order and identities are stable across evidence selection"
        );
        assert_eq!(default_method.text, all_method.text);
    }

    let default_method = method(&default, b"rawAliasEscape");
    let all_method = method(&all, b"rawAliasEscape");
    let default_raw = raw_body(default_method);
    let all_raw = raw_body(all_method);
    assert_eq!(default_method.text, all_method.text);
    assert_eq!(default_raw.text, all_raw.text);
    assert_eq!(default_raw.source_map, all_raw.source_map);

    assert!(
        default_raw
            .text
            .contains("java.lang.Integer value = (java.lang.Integer) local4.next();"),
        "the first reused slot keeps its Integer type and cast in the loop declaration: {}",
        default_raw.text
    );
    assert!(
        default_raw
            .text
            .contains("java.lang.String value_2 = (java.lang.String) local4.next();"),
        "the second reused slot keeps its String type, cast and stable collision suffix in the loop declaration: {}",
        default_raw.text
    );
    assert!(default_raw.text.contains("escaped = target;"));
    assert!(
        default_raw
            .text
            .contains("escaped.add((java.lang.Object) value)"),
        "the escaped raw List alias and Object insertion remain visible: {}",
        default_raw.text
    );
    assert!(
        default_raw.text.contains("(java.lang.Integer)")
            && default_raw.text.contains("(java.lang.String)"),
        "both original checkcasts remain in raw recovery: {}",
        default_raw.text
    );
    assert_utf8_source_map(default_raw);
    assert_bcis_are_mapped(default_raw, &[30, 35, 72, 77]);
}

#[test]
fn original_nested_loops_folds_both_proven_local_bindings_without_changing_raw_sources() {
    let class_name = "jadx/tests/integration/loops/TestNestedLoops$TestCls";
    for (bytes, outer_name, inner_name, first, second) in [
        (NESTED_LOOPS_DEBUG, "s1", "s2", "l1", "l2"),
        (NESTED_LOOPS_NONE, "local4", "local6", "arg1", "arg2"),
    ] {
        let default = performed(bytes, class_name);
        let all = match report_with(
            bytes,
            class_name,
            Some(&RecoveryEvidenceRequest::all()),
            &mut task_budget(&[]).unwrap(),
        ) {
            OperationOutcome::Performed(report) => report,
            other => panic!("all-evidence original nested-loop class completes: {other:?}"),
        };
        let no_rule_details_request = RecoveryEvidenceRequest::essential()
            .with_kind(RecoveryEvidenceKind::SourceMap)
            .with_kind(RecoveryEvidenceKind::RegionDetails)
            .with_kind(RecoveryEvidenceKind::NameDetails);
        let no_rule_details = match report_with(
            bytes,
            class_name,
            Some(&no_rule_details_request),
            &mut task_budget(&[]).unwrap(),
        ) {
            OperationOutcome::Performed(report) => report,
            other => panic!("no-RuleDetails original nested-loop class completes: {other:?}"),
        };
        assert_eq!(default.text, all.text);
        assert_eq!(default.text, no_rule_details.text);

        let facts = class_facts(bytes, &mut task_budget(&[]).unwrap()).unwrap();
        for name in [b"<init>".as_slice(), b"test"] {
            let d = method(&default, name);
            let a = method(&all, name);
            let n = method(&no_rule_details, name);
            assert_eq!(d.item.identity, a.item.identity);
            assert_eq!(d.item.identity, n.item.identity);
            assert_eq!(d.text, a.text, "{}", d.text);
            assert_eq!(d.text, n.text, "{}", d.text);
            let raw = raw_body(d);
            let raw_all = raw_body(a);
            let raw_plain = raw_body(n);
            assert_eq!(raw.text, raw_all.text, "{}", d.text);
            assert_eq!(raw.text, raw_plain.text, "{}", d.text);
            assert_eq!(raw.source_map, raw_all.source_map);
            assert_eq!(raw.source_map, raw_plain.source_map);
            assert_eq!(raw.representation, Representation::Java, "{}", raw.text);
            assert_eq!(raw.quality, Quality::Structured, "{}", raw.text);
            assert_eq!(
                raw_all.evidence.state(RecoveryEvidenceKind::RuleDetails),
                EvidenceState::Complete
            );
            assert_eq!(
                raw_plain.evidence.state(RecoveryEvidenceKind::RuleDetails),
                EvidenceState::NotRequested
            );
            assert_utf8_source_map(raw);
            assert_raw_map_method_identity(raw, d);
            assert_all_instruction_bcis_are_mapped(bytes, &facts, d);
        }

        let d = method(&default, b"test");
        let projected = &d.text;
        assert!(
            projected.contains("test(java.util.List<java.lang.String>"),
            "the List<String> source signature remains: {projected}"
        );
        for (name, values) in [(outer_name, first), (inner_name, second)] {
            let header = format!("for (java.lang.String {name} : {values}) {{");
            assert!(
                projected.lines().any(|line| line.trim() == header),
                "expected existing local in header `{header}`: {projected}"
            );
        }
        for removed in [
            format!("java.lang.String {outer_name};"),
            format!("{outer_name} = iteratorElement17;"),
            format!("java.lang.String {inner_name} = iteratorElement47;"),
        ] {
            assert!(
                !projected.lines().any(|line| line.trim() == removed),
                "binding copy/declaration should be folded: {removed}: {projected}"
            );
        }
        assert!(
            !projected.contains("iteratorElement"),
            "both old synthetic element names are gone from the projection: {projected}"
        );
        assert!(
            !projected.contains("java.util.Iterator local5;"),
            "the fully consumed ancestor iterator declaration is removed: {projected}"
        );
        assert!(
            projected.contains(&format!("{outer_name}.equals({inner_name})")),
            "the proved String equals argument omits its Object wrapper: {projected}"
        );
        assert!(
            projected.contains(&format!("{second}.add({outer_name});")),
            "the original List insertion body remains: {projected}"
        );
        assert!(
            projected.contains(&format!("{first}.remove({inner_name});")),
            "the proved reference remove argument omits its Object wrapper: {projected}"
        );
        assert!(
            projected.contains(&format!("if ({second}.size() > 0) {{"))
                && projected.contains(&format!("{first}.clear();")),
            "the statements after both loops remain: {projected}"
        );

        let raw = raw_body(d);
        assert!(
            !raw.text
                .lines()
                .any(|line| line.trim() == "java.util.Iterator local5;"),
            "the raw body also has no exact blank ancestor iterator declaration: {}",
            raw.text
        );
        assert!(
            raw.text.contains("for (java.lang.Object iteratorElement17"),
            "{}",
            raw.text
        );
        assert!(
            raw.text.contains("for (java.lang.Object iteratorElement47"),
            "{}",
            raw.text
        );
        assert!(
            raw.text.contains("(java.lang.String) iteratorElement17"),
            "{}",
            raw.text
        );
        assert!(
            raw.text.contains("(java.lang.String) iteratorElement47"),
            "{}",
            raw.text
        );
        assert!(
            raw.text
                .contains(&format!("{second}.add((java.lang.Object) {outer_name})")),
            "{}",
            raw.text
        );

        let constructor = method(&default, b"<init>");
        assert!(
            constructor.text.contains("super();"),
            "{}",
            constructor.text
        );
        assert!(constructor.text.contains("return;"), "{}", constructor.text);
    }
}

#[test]
fn typed_list_foreach_element_heads_preserve_bindings_and_raw_origins() {
    let class_name = "typedforeach/TypedListForEachCases";
    let default = performed(TYPED_LIST_FOREACH, class_name);
    let all = match report_with(
        TYPED_LIST_FOREACH,
        class_name,
        Some(&RecoveryEvidenceRequest::all()),
        &mut task_budget(&[]).unwrap(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("all-evidence typed foreach class completes: {other:?}"),
    };
    let essential_with_map = RecoveryEvidenceRequest::essential()
        .with_kind(RecoveryEvidenceKind::SourceMap)
        .with_kind(RecoveryEvidenceKind::RegionDetails)
        .with_kind(RecoveryEvidenceKind::NameDetails);
    let essential = match report_with(
        TYPED_LIST_FOREACH,
        class_name,
        Some(&essential_with_map),
        &mut task_budget(&[]).unwrap(),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("essential-evidence typed foreach class completes: {other:?}"),
    };
    assert_eq!(default.text, all.text);
    assert_eq!(default.text, essential.text);

    let facts = class_facts(TYPED_LIST_FOREACH, &mut task_budget(&[]).unwrap()).unwrap();
    for name in [
        b"strings".as_slice(),
        b"integers",
        b"raw",
        b"mismatch",
        b"declared",
    ] {
        let d = method(&default, name);
        let a = method(&all, name);
        let e = method(&essential, name);
        assert_eq!(d.text, a.text, "{}", d.text);
        assert_eq!(d.text, e.text, "{}", d.text);
        let raw = raw_body(d);
        let raw_all = raw_body(a);
        let raw_essential = raw_body(e);
        assert_eq!(raw.text, raw_all.text, "{}", d.text);
        assert_eq!(raw.text, raw_essential.text, "{}", d.text);
        assert_eq!(raw.representation, Representation::Java, "{}", d.text);
        assert_eq!(raw.quality, Quality::Structured, "{}", d.text);
        assert_eq!(raw.source_map, raw_all.source_map);
        assert_eq!(raw.source_map, raw_essential.source_map);
        assert_eq!(
            raw_essential
                .evidence
                .state(RecoveryEvidenceKind::RuleDetails),
            EvidenceState::NotRequested
        );
        assert_utf8_source_map(raw);
        assert_raw_map_method_identity(raw, d);
        assert_all_instruction_bcis_are_mapped(TYPED_LIST_FOREACH, &facts, d);
    }

    let strings = &method(&default, b"strings").text;
    assert!(
        strings.contains("strings(java.util.List<java.lang.String> values)"),
        "{strings}"
    );
    assert_eq!(
        strings
            .lines()
            .filter(|line| line.trim_start().starts_with("for (java.lang.String "))
            .count(),
        1,
        "{strings}"
    );
    let string_head = strings
        .lines()
        .find(|line| line.trim_start().starts_with("for (java.lang.String "))
        .unwrap()
        .trim_start()
        .split(" : ")
        .next()
        .unwrap()
        .strip_prefix("for (java.lang.String ")
        .unwrap();
    assert!(strings.contains("java.lang.String current;"), "{strings}");
    assert!(strings.contains("current = \"before\";"), "{strings}");
    assert!(
        strings.contains(&format!("current = {string_head};")),
        "preserve the loop-header copy: {strings}"
    );
    assert!(
        strings.contains("typedforeach.TypedListForEachCases.after = current;"),
        "{strings}"
    );
    assert!(
        !strings.contains("(java.lang.String)"),
        "typed header consumes the redundant cast: {strings}"
    );

    let integers = &method(&default, b"integers").text;
    assert!(
        integers.contains("integers(java.util.List<java.lang.Integer> values)"),
        "{integers}"
    );
    assert_eq!(
        integers
            .lines()
            .filter(|line| line.trim_start().starts_with("for (java.lang.Integer "))
            .count(),
        1,
        "{integers}"
    );
    let integer_head = integers
        .lines()
        .find(|line| line.trim_start().starts_with("for (java.lang.Integer "))
        .unwrap()
        .trim_start()
        .split(" : ")
        .next()
        .unwrap()
        .strip_prefix("for (java.lang.Integer ")
        .unwrap();
    assert!(integers.contains("Object current;"), "{integers}");
    assert!(integers.contains("current = null;"), "{integers}");
    assert!(
        integers.contains(&format!("current = {integer_head};")),
        "preserve the loop-header copy: {integers}"
    );
    assert!(
        integers.contains("typedforeach.TypedListForEachCases.after = current;"),
        "{integers}"
    );
    assert!(
        !integers.contains("(java.lang.Integer)"),
        "typed header consumes the redundant cast: {integers}"
    );

    let declared = &method(&default, b"declared").text;
    assert_eq!(
        declared
            .lines()
            .filter(|line| line.trim_start().starts_with("for (java.lang.String "))
            .count(),
        1,
        "{declared}"
    );
    assert!(
        declared.contains("for (java.lang.String value : values) {"),
        "the existing local name is promoted to the typed header: {declared}"
    );
    assert!(
        !declared.contains("java.lang.String value ="),
        "the first body copy is folded into the header: {declared}"
    );
    assert!(
        declared.contains("typedforeach.TypedListForEachCases.seen = value;"),
        "{}",
        declared
    );
    assert!(
        !declared.contains(".add(") && !declared.contains("remove("),
        "typed foreach is the only body projection: {declared}"
    );

    for (name, object_head, cast_type) in [
        (
            b"strings".as_slice(),
            "java.lang.Object",
            "java.lang.String",
        ),
        (
            b"integers".as_slice(),
            "java.lang.Object",
            "java.lang.Integer",
        ),
        (
            b"declared".as_slice(),
            "java.lang.Object",
            "java.lang.String",
        ),
    ] {
        let member = method(&default, name);
        let raw = raw_body(member);
        assert!(
            raw.text.contains(&format!("for ({object_head} ")),
            "{}",
            raw.text
        );
        assert!(raw.text.contains(&format!("({cast_type})")), "{}", raw.text);
        let code = method_code(&facts, TYPED_LIST_FOREACH, member);
        let cast = code
            .instructions
            .iter()
            .find(|i| i.opcode == 0xc0)
            .expect("original foreach element has a physical checkcast");
        assert_bci_maps_to(member, cast.bci, cast_type);
    }

    let raw_method = method(&default, b"raw");
    assert!(
        raw_method.text.contains("raw(java.util.List values)"),
        "{}",
        raw_method.text
    );
    assert!(
        raw_method.text.contains("for (java.lang.Object "),
        "{}",
        raw_method.text
    );
    assert!(
        raw_method.text.contains("(java.lang.String)"),
        "{}",
        raw_method.text
    );
    let mismatch = method(&default, b"mismatch");
    assert!(
        mismatch
            .text
            .contains("mismatch(java.util.List<java.lang.String> values)"),
        "{}",
        mismatch.text
    );
    assert!(
        mismatch.text.contains("for (java.lang.Object "),
        "{}",
        mismatch.text
    );
    assert!(
        mismatch.text.contains("(java.lang.Integer)"),
        "{}",
        mismatch.text
    );
}

#[derive(Debug)]
struct DeadlineObserver {
    entries: AtomicUsize,
    delay_at: usize,
    delayed: std::sync::atomic::AtomicBool,
}

impl LedgerObserver for DeadlineObserver {
    fn entry(&self, _site: LedgerSite, _timing: Option<LedgerEntryTiming>) {
        let observed = self.entries.fetch_add(1, Ordering::SeqCst) + 1;
        if observed == self.delay_at && !self.delayed.swap(true, Ordering::SeqCst) {
            std::thread::sleep(std::time::Duration::from_millis(1_100));
        }
    }

    fn refusal(&self, _site: LedgerSite, _dimension: Option<BudgetDimension>) {}
}

fn observed_typed_foreach_report(
    cancellation: CancellationToken,
    observer: Arc<EntryObserver>,
) -> Result<OperationOutcome<ClassSourceReport>> {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(TYPED_LIST_FOREACH.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = request(&snapshot, "typedforeach/TypedListForEachCases");
    let mut budget = Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation);
    let ledger = OperationLedger::new(&budget).with_observer(observer);
    budget.with_ledger(ledger, UsageOwner::Methods);
    engine.class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
}

#[test]
fn typed_foreach_late_deadline_keeps_raw_artifact_without_partial_projection() {
    let token = CancellationToken::new();
    let complete_observer = Arc::new(EntryObserver::new(None, token.clone()));
    let complete = match observed_typed_foreach_report(token, complete_observer.clone()) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("observed typed-foreach run must complete: {other:?}"),
    };
    assert!(matches!(
        &complete.execution,
        ExecutionReport::Complete { .. }
    ));
    let complete_strings = method(&complete, b"strings");
    assert_eq!(
        complete
            .text
            .lines()
            .filter(|line| line.trim_start().starts_with("for (java.lang.String "))
            .count(),
        2,
        "the complete class installs both String foreach heads: {}",
        complete.text
    );
    let complete_raw = raw_body(complete_strings);

    // Find the first boundary that commits the strings projection. The immediately preceding
    // boundary is delayed once so the elapsed-time check fires before that staged projection can commit.
    let mut lower = 1;
    let mut upper = complete_observer.count();
    while lower + 1 < upper {
        let boundary = lower + (upper - lower) / 2;
        let token = CancellationToken::new();
        let observer = Arc::new(EntryObserver::new(Some(boundary), token.clone()));
        let committed = match observed_typed_foreach_report(token, observer) {
            Ok(OperationOutcome::Performed(report)) => report.methods.iter().any(|member| {
                member.item.name.raw().0 == b"strings"
                    && member.text.contains("for (java.lang.String ")
            }),
            Ok(OperationOutcome::Incomplete(_)) | Err(Error::Cancelled { .. }) => false,
            Ok(OperationOutcome::Ambiguous(_)) => panic!("frozen class must bind uniquely"),
            Err(error) => panic!("unexpected boundary-search failure: {error:?}"),
        };
        if committed {
            upper = boundary;
        } else {
            lower = boundary;
        }
    }

    let token = CancellationToken::new();
    let observer = Arc::new(DeadlineObserver {
        entries: AtomicUsize::new(0),
        delay_at: lower,
        delayed: std::sync::atomic::AtomicBool::new(false),
    });
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(TYPED_LIST_FOREACH.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = request(&snapshot, "typedforeach/TypedListForEachCases");
    let mut limits = task_limits(&[]).unwrap();
    limits.elapsed_millis = 1_000;
    let mut budget = Budget::with_cancellation_token(limits, token);
    let ledger = OperationLedger::new(&budget).with_observer(observer.clone());
    budget.with_ledger(ledger, UsageOwner::Methods);
    let report = match engine.class_source(std::slice::from_ref(&snapshot), &request, &mut budget) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("deadline stop must retain the class report: {other:?}"),
    };
    assert!(
        observer.delayed.load(Ordering::SeqCst),
        "deadline delay was reached"
    );
    assert!(matches!(
        &report.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
            },
            ..
        }
    ));
    let stopped_strings = method(&report, b"strings");
    assert!(
        !stopped_strings.text.contains("for (java.lang.String "),
        "{}",
        stopped_strings.text
    );
    assert!(
        !stopped_strings
            .text
            .contains("strings(java.util.List<java.lang.String>"),
        "{}",
        stopped_strings.text
    );
    let stopped_raw = raw_body(stopped_strings);
    assert_eq!(stopped_raw.text, complete_raw.text);
    assert_eq!(stopped_raw.source_map, complete_raw.source_map);
    assert_utf8_source_map(stopped_raw);
}

fn observed_nested_loops_report(
    cancellation: CancellationToken,
    observer: Arc<EntryObserver>,
) -> Result<OperationOutcome<ClassSourceReport>> {
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(NESTED_LOOPS_DEBUG.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = request(
        &snapshot,
        "jadx/tests/integration/loops/TestNestedLoops$TestCls",
    );
    let mut budget = Budget::with_cancellation_token(task_limits(&[]).unwrap(), cancellation);
    let ledger = OperationLedger::new(&budget).with_observer(observer);
    budget.with_ledger(ledger, UsageOwner::Methods);
    engine.class_source(std::slice::from_ref(&snapshot), &request, &mut budget)
}

fn nested_loops_both_heads_are_folded(report: &ClassSourceReport) -> bool {
    let method = report
        .methods
        .iter()
        .find(|member| member.item.name.raw().0 == b"test");
    method.is_some_and(|member| {
        member.text.contains("for (java.lang.String s1 : l1)")
            && member.text.contains("for (java.lang.String s2 : l2)")
    })
}

fn assert_nested_loops_no_partial_fold(stopped: &ClassSourceReport, complete_raw: &RecoveryReport) {
    let method = method(stopped, b"test");
    assert!(
        !method.text.contains("for (java.lang.String "),
        "neither staged typed head nor optional fold may publish: {}",
        method.text
    );
    assert!(
        !method.text.contains("for (java.lang.String s1 : l1)"),
        "outer fold was not published partially: {}",
        method.text
    );
    assert!(
        !method.text.contains("for (java.lang.String s2 : l2)"),
        "inner fold was not published partially: {}",
        method.text
    );
    let generic_declaration =
        "test(java.util.List<java.lang.String> l1, java.util.List<java.lang.String> l2)";
    assert!(
        !stopped.text.contains(generic_declaration),
        "{}",
        stopped.text
    );
    assert!(
        !method.text.contains(generic_declaration),
        "{}",
        method.text
    );
    let raw = raw_body(method);
    assert_eq!(raw.text, complete_raw.text);
    assert_eq!(raw.source_map, complete_raw.source_map);
    assert_utf8_source_map(raw);
}

#[test]
fn nested_foreach_late_cancel_budget_and_deadline_keep_raw_source_atomically() {
    let complete_token = CancellationToken::new();
    let complete_observer = Arc::new(EntryObserver::new(None, complete_token.clone()));
    let complete = match observed_nested_loops_report(complete_token, complete_observer.clone()) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("observed nested-loop run must complete: {other:?}"),
    };
    assert!(matches!(
        &complete.execution,
        ExecutionReport::Complete { .. }
    ));
    assert!(
        nested_loops_both_heads_are_folded(&complete),
        "{}",
        complete.text
    );
    let complete_raw = raw_body(method(&complete, b"test"));

    // Find the first observer boundary that has committed both nested folded headers.
    let mut cancel_lower = 0;
    let mut upper = complete_observer.count();
    assert!(upper > 1, "the target has observable ledger boundaries");
    while cancel_lower + 1 < upper {
        let boundary = cancel_lower + (upper - cancel_lower) / 2;
        let token = CancellationToken::new();
        let observer = Arc::new(EntryObserver::new(Some(boundary), token.clone()));
        let committed = match observed_nested_loops_report(token, observer) {
            Ok(OperationOutcome::Performed(report)) => nested_loops_both_heads_are_folded(&report),
            Ok(OperationOutcome::Incomplete(_)) | Err(Error::Cancelled { .. }) => false,
            Ok(OperationOutcome::Ambiguous(_)) => panic!("frozen class must bind uniquely"),
            Err(error) => panic!("unexpected cancellation-boundary failure: {error:?}"),
        };
        if committed {
            upper = boundary;
        } else {
            cancel_lower = boundary;
        }
    }
    let cancel_token = CancellationToken::new();
    let cancel_observer = Arc::new(EntryObserver::new(Some(cancel_lower), cancel_token.clone()));
    let cancelled = match observed_nested_loops_report(cancel_token, cancel_observer) {
        Ok(OperationOutcome::Performed(report)) => report,
        other => panic!("late cancellation must retain the class report: {other:?}"),
    };
    assert!(matches!(
        &cancelled.execution,
        ExecutionReport::Cancelled { .. }
    ));
    assert_nested_loops_no_partial_fold(&cancelled, &complete_raw);

    // Find the corresponding first successful AnalysisSteps boundary, then stop one step before it.
    let mut budget_lower = 0;
    let mut upper = complete.usage.analysis_steps;
    while budget_lower + 1 < upper {
        let boundary = budget_lower + (upper - budget_lower) / 2;
        let mut limits = task_limits(&[]).unwrap();
        limits.analysis_steps = boundary;
        let committed = match report_with(
            NESTED_LOOPS_DEBUG,
            "jadx/tests/integration/loops/TestNestedLoops$TestCls",
            None,
            &mut Budget::new(limits),
        ) {
            OperationOutcome::Performed(report) => nested_loops_both_heads_are_folded(&report),
            _ => false,
        };
        if committed {
            upper = boundary;
        } else {
            budget_lower = boundary;
        }
    }
    let mut limits = task_limits(&[]).unwrap();
    limits.analysis_steps = budget_lower;
    let budget_stopped = match report_with(
        NESTED_LOOPS_DEBUG,
        "jadx/tests/integration/loops/TestNestedLoops$TestCls",
        None,
        &mut Budget::new(limits),
    ) {
        OperationOutcome::Performed(report) => report,
        other => panic!("late analysis-budget stop must retain the class report: {other:?}"),
    };
    assert!(matches!(
        &budget_stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::AnalysisSteps,
            },
            ..
        }
    ));
    assert_nested_loops_no_partial_fold(&budget_stopped, &complete_raw);

    // Delay at the immediately preceding ledger boundary so the deadline expires before commit.
    assert!(
        cancel_lower > 0,
        "the first fold commit has a preceding ledger boundary"
    );
    let deadline_observer = Arc::new(DeadlineObserver {
        entries: AtomicUsize::new(0),
        delay_at: cancel_lower,
        delayed: std::sync::atomic::AtomicBool::new(false),
    });
    let engine = Engine::new();
    let snapshot = engine
        .open(
            ArtifactInput::bytes(NESTED_LOOPS_DEBUG.to_vec()),
            &mut task_budget(&[]).unwrap(),
        )
        .unwrap();
    let request = request(
        &snapshot,
        "jadx/tests/integration/loops/TestNestedLoops$TestCls",
    );
    let mut limits = task_limits(&[]).unwrap();
    limits.elapsed_millis = 1_000;
    let mut budget = Budget::new(limits);
    let ledger = OperationLedger::new(&budget).with_observer(deadline_observer.clone());
    budget.with_ledger(ledger, UsageOwner::Methods);
    let deadline_stopped =
        match engine.class_source(std::slice::from_ref(&snapshot), &request, &mut budget) {
            Ok(OperationOutcome::Performed(report)) => report,
            other => panic!("late deadline must retain the class report: {other:?}"),
        };
    assert!(
        deadline_observer.delayed.load(Ordering::SeqCst),
        "deadline delay was reached"
    );
    assert!(matches!(
        &deadline_stopped.execution,
        ExecutionReport::Partial {
            reason: TerminationReason::BudgetExceeded {
                dimension: BudgetDimension::ElapsedMillis,
            },
            ..
        }
    ));
    assert_nested_loops_no_partial_fold(&deadline_stopped, &complete_raw);
}
