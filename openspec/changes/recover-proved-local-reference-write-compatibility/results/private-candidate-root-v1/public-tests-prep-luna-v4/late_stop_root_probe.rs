//! Temporary root measurement snippet; not part of the permanent public API suite.
//! Paste/use only after the public test helpers are available. No numeric usage is pinned.

fn late_stop_measurement_probe(
    bytes: &[u8],
    internal_name: &str,
    method_name: &str,
    descriptor: &str,
) -> (u64, OperationOutcome<ClassSourceReport>) {
    let (snapshot, request) = opened(bytes, internal_name);
    let complete = recover(&snapshot, &request, &RecoveryEvidenceRequest::all());
    let (_, body) = named_method(&complete, method_name, descriptor);
    let ExecutionReport::Complete { usage } = &body.execution else {
        panic!("probe requires an observed complete method: {:?}", body.execution);
    };
    let stop_limit = usage.analysis_steps.saturating_sub(1);
    let mut limits = budget().limits().clone();
    limits.analysis_steps = stop_limit;
    let outcome = Engine::new()
        .class_source_with_evidence(
            std::slice::from_ref(&snapshot),
            &request,
            &RecoveryEvidenceRequest::all(),
            &mut Budget::new(limits),
        )
        .expect("late-stop probe returns the ordinary execution state");
    (stop_limit, outcome)
}
