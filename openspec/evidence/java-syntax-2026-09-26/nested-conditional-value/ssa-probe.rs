use jarde_jvm::{analyze_method_ir, ir::{AnalysisStage, MethodAnalysisRequest}, method_ir::{Definition, PhiInput}};
use jarde_reader::{artifact::{ArtifactInput, ArtifactSnapshot}, budget::{Budget, Limits}, model::PhysicalMethodId, view::{LayoutMode, LoaderId, MultiReleasePolicy, PhysicalScope, RuntimeProfile}};
use std::{env, path::PathBuf};
use jarde::{EnvironmentPolicy, EnvironmentRequest};
fn main() {
    let a: Vec<String> = env::args().collect();
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(&a[2]).unwrap()).unwrap();
    let method: PhysicalMethodId = serde_json::from_value(report["method"].clone()).unwrap();
    let limits = Limits { input_bytes: 100_000_000, archive_entries: 10_000, entry_bytes: 100_000_000, read_bytes: 100_000_000, class_bytes: 100_000_000, attribute_bytes: 100_000_000, code_bytes: 100_000_000, result_items: 100_000, output_bytes: 100_000_000, class_headers: 100, method_bodies: 100, ir_items: 1_000_000, ir_edges: 1_000_000, analysis_steps: 10_000_000, normalization_clones: 100_000, nested_depth: 10, dependency_depth: 10, elapsed_millis: 30_000 };
    let mut budget = Budget::new(limits);
    let snapshot = ArtifactSnapshot::open(ArtifactInput::Path(PathBuf::from(&a[1])), &mut budget).unwrap();
    let env = EnvironmentRequest { snapshot: snapshot.id().clone(), scope: PhysicalScope::SnapshotAll, policy: EnvironmentPolicy::SingleClass, profile: RuntimeProfile { java_release: 8, multi_release: MultiReleasePolicy::Disabled, layout: LayoutMode::Generic }, loader: LoaderId("app".into()) }.build(std::slice::from_ref(&snapshot)).unwrap();
    let request = MethodAnalysisRequest { environment: env, method, stages: AnalysisStage::ALL.to_vec() };
    let analyzed = analyze_method_ir(std::slice::from_ref(&snapshot), &request, &mut budget).unwrap();
    let ir = analyzed.ir();
    let cfg = ir.canonical().unwrap();
    println!("blocks:");
    for b in cfg.blocks() { println!("  BCI {} blocks {:?}", b.id().bci(), b.blocks()); }
    println!("normal edges:");
    for e in cfg.edges().iter().filter(|e| format!("{:?}", e.kind()) == "Normal") { println!("  {} -> {}", e.from().bci(), e.to().bci()); }
    println!("phis:");
    for phi in ir.ssa().unwrap().phis() {
        let ins: Vec<String> = phi.inputs().iter().map(|i| match i { PhiInput::Itself => "Itself".into(), PhiInput::Value(v) => {let def=ir.ssa().unwrap().value(*v).def(); format!("{:?}", def)} }).collect();
        println!("  block={} slot={:?} inputs={:?}", phi.block().bci(), phi.slot(), ins);
    }
    println!("values at block join:");
    for (id,v) in ir.ssa().unwrap().values_with_ids() { if matches!(v.def(), Definition::Phi { .. } | Definition::Entry { .. }) { println!("  {:?} {:?} replaced={:?}", id, v.def(), v.replaced_by()); } }
}
