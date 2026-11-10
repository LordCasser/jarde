//! Measurement adapter only; compiled as a temporary integration target against S0 code with test-only lifetime/work observations.
use jarde::*;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

fn value<T: serde::Serialize>(x: &T) -> Value { serde_json::to_value(x).unwrap() }
fn counters(a: jarde::d0_counts::Counts) -> Value {
    let d = a.since(jarde::d0_counts::snapshot());
    json!({"class_materializations":d.class_materializations,"class_preparations":d.class_preparations,
        "body_decodes":d.body_decodes,"recovery_runs":d.recovery_runs,
        "owned_records":d.owned_records,"read_detail_records":d.read_detail_records})
}
fn budget(limits: &Limits, cache: &Option<FactsCache>) -> Budget {
    let b = Budget::new(limits.clone());
    match cache { Some(c) => b.with_facts_cache(c.clone()), None => b }
}
struct Sink {
    hash: blake3::Hasher,
    methods: u64,
    events: Vec<Value>,
    prepared: BTreeSet<u64>,
    missing_prepared: Vec<u64>,
    max_text_bytes: usize,
    max_owned_weight: u64,
    mode: String,
    signal: Option<mpsc::Sender<()>>,
    delivery: Option<DeliveryAccount>,
    encoded_peak: usize,
    encoded_total: u64,
}
impl Sink {
    fn event(&mut self, v: Value) { if self.events.len() < 256 { self.events.push(v); } }
}
impl RecoverySink for Sink {
    fn header(&mut self, e: &BulkHeaderEvent, d: DeliveryAccount) -> Result<SinkControl> {
        self.delivery = Some(d); self.event(json!({"kind":"header","limits":e.limits}));
        Ok(SinkControl::Continue)
    }
    fn class_prepared(&mut self, e: &ClassPreparedEvent) -> Result<SinkControl> {
        self.prepared.insert(e.class_ordinal);
        self.event(json!({"kind":"prepared","class_ordinal":e.class_ordinal}));
        if let Some(s) = self.signal.take() { let _ = s.send(()); }
        Ok(SinkControl::Continue)
    }
    fn method(&mut self, e: &MethodResultEvent) -> Result<SinkControl> {
        self.methods += 1;
        self.hash.update(&serde_json::to_vec(&e.method).unwrap());
        self.hash.update(&serde_json::to_vec(&e.outcome()).unwrap());
        if let MethodDelivery::Recovered(r) = &e.delivery {
            self.hash.update(r.recovery().text.as_bytes());
            self.max_text_bytes = self.max_text_bytes.max(r.recovery().text.len());
        }
        self.max_owned_weight = self.max_owned_weight.max(e.weight);
        self.event(json!({"kind":"method","class_ordinal":e.class_ordinal,"member_ordinal":e.member_ordinal}));
        if self.mode == "slow" {
            let bytes = serde_json::to_vec(e).unwrap();
            self.delivery.as_ref().unwrap().charge_output_bytes(bytes.len() as u64)?;
            self.encoded_peak = self.encoded_peak.max(bytes.capacity());
            self.encoded_total += bytes.len() as u64;
            std::thread::sleep(Duration::from_millis(2));
        }
        if self.mode == "sink" && self.methods == 2 { return Ok(SinkControl::Stop); }
        Ok(SinkControl::Continue)
    }
    fn class_end(&mut self, e: &ClassEndEvent) -> Result<SinkControl> {
        if !self.prepared.contains(&e.class_ordinal) && !matches!(e.completion, ClassCompletion::Refused {..}) {
            self.missing_prepared.push(e.class_ordinal);
        }
        self.event(json!({"kind":"class_end","event":e})); Ok(SinkControl::Continue)
    }
    fn diagnostic(&mut self, e: &BulkDiagnosticEvent) -> Result<SinkControl> {
        self.event(json!({"kind":"diagnostic","event":e})); Ok(SinkControl::Continue)
    }
    fn final_event(&mut self, e: &BulkFinalEvent) -> Result<SinkControl> {
        self.event(json!({"kind":"final","event":e})); Ok(SinkControl::Continue)
    }
}

fn performed<T>(o: OperationOutcome<T>) -> T {
    match o { OperationOutcome::Performed(x) => x, _ => panic!("frozen target did not bind") }
}

fn operation(cfg: &Value, engine: &Engine, snapshot: &ArtifactSnapshot,
    env: &EnvironmentRequest, cache: &Option<FactsCache>, limits: &Limits,
    evidence: &RecoveryEvidenceRequest) -> Value {
    jarde_reader::observation::reset_peaks();
    jarde_java::observation::reset_peaks();
    let reader_before = jarde_reader::observation::snapshot();
    let jvm_before = jarde_jvm::observation::snapshot();
    let java_before = jarde_java::observation::snapshot();
    let before = jarde::d0_counts::snapshot();
    let started = Instant::now();
    let mut b = budget(limits, cache);
    let mut text_bytes = 0;
    let mut hash = blake3::Hasher::new();
    let mut extra = json!({});
    let shape = cfg["shape"].as_str().unwrap();
    if shape == "single" {
        let r = performed(engine.recover_target_with_evidence(std::slice::from_ref(snapshot),
            &MethodOperationRequest { method:MethodRef::Method { method:serde_json::from_value(cfg["method"].clone()).unwrap() }, environment:env.clone() },
            evidence, &mut b).unwrap());
        let core = r.recovered.recovery(); text_bytes = core.text.len(); hash.update(core.text.as_bytes());
        extra = json!({"outcome":core.outcome,"quality":core.quality,"execution":core.execution,"representation":core.representation});
    } else if shape == "class" {
        let r = performed(engine.class_source_with_evidence(std::slice::from_ref(snapshot),
            &ClassSourceRequest { class:ClassRef::Definition { definition:serde_json::from_value(cfg["method"]["owner"].clone()).unwrap() },environment:env.clone() },
            evidence, &mut b).unwrap());
        text_bytes = r.text.len(); hash.update(r.text.as_bytes());
        extra = json!({"execution":r.execution,"methods":r.methods.len()});
    } else if shape == "query" {
        let mut items = Vec::new();
        for _ in 0..6 {
            let q = QueryRequest { relation:QueryRelation::ConstantPoolContains,
                target:QueryTarget::Symbol { value:SymbolRef::Class { owner:JvmBytes(b"java/lang/Object".to_vec()) } },
                physical:PhysicalView { snapshot:snapshot.id().clone(),scope:env.scope.clone() },
                consumers:ConsumerSchema::new(1,vec![ConsumerKind::Type]), max_items:2, cursor:None };
            let r = engine.query(snapshot,&q,&mut b).unwrap();
            let encoded = serde_json::to_vec(&r.items).unwrap(); hash.update(&encoded); text_bytes += encoded.len();
            items.push(json!({"items":r.items.len(),"has_more":r.page.has_more,"execution":r.execution}));
        }
        extra = json!({"requests":items,"note":"six fresh first-page queries, not full-scope exhaustion"});
    } else {
        let mode = cfg["fault"].as_str().unwrap_or("normal");
        let workers = cfg["workers"].as_u64().unwrap_or(1) as usize;
        let watch = Arc::new(WorkerWatch::new());
        let probe = Arc::new(BulkProbe::new());
        let token = CancellationToken::new();
        let trigger = Arc::new(Mutex::new(None));
        let mut request = BulkRecoveryRequest::for_scope(env.clone(),workers,serde_json::from_value(cfg["method_limits"].clone()).unwrap()).with_evidence(evidence.clone()).with_probe(probe.clone());
        request.faults.worker_watch = Some(watch.clone());
        if mode == "panic" { request.faults.panic_at_class_ordinal = Some(1); }
        if mode == "refused" { request.max_class_bytes = 1; }
        if mode == "slow" { request.max_result_weight = 1<<17; request.max_buffered_result_weight = 1<<18; }
        let (send,recv) = mpsc::channel();
        let thread = if mode == "token" {
            // Wait for real preparation, then cancel concurrently while the request is active.
            let t=token.clone(); let clock=trigger.clone();
            Some(std::thread::spawn(move || { if recv.recv_timeout(Duration::from_secs(10)).is_ok() {
                std::thread::sleep(Duration::from_millis(1));
                let mut slot=clock.lock().unwrap();
                let reader_before_set=jarde_reader::observation::snapshot();
                let jvm_before_set=jarde_jvm::observation::snapshot();
                let at=Instant::now(); t.cancel();
                *slot=Some((at,jarde::d0_counts::snapshot(),reader_before_set,jvm_before_set,jarde_reader::observation::snapshot(),jarde_jvm::observation::snapshot()));
            }}))
        } else { None };
        if mode == "token" { b=Budget::with_cancellation_token(limits.clone(),token.clone()); if let Some(c)=cache {b=b.with_facts_cache(c.clone());} }
        if mode == "budget" { let mut l=limits.clone();l.method_bodies=1;b=budget(&l,cache); }
        if mode == "elapsed" { let mut l=limits.clone();l.elapsed_millis=1;b=budget(&l,cache); std::thread::sleep(Duration::from_millis(3)); }
        let mut sink=Sink { hash:blake3::Hasher::new(),methods:0,events:Vec::new(),prepared:BTreeSet::new(),missing_prepared:Vec::new(),max_text_bytes:0,max_owned_weight:0,mode:mode.into(),signal:if mode=="token" {Some(send)} else {None},delivery:None,encoded_peak:0,encoded_total:0 };
        let report=engine.recover_all(std::slice::from_ref(snapshot),&request,&mut b,&mut sink);
        let returned=Instant::now();
        if let Some(h)=thread { h.join().unwrap(); }
        let token_data=trigger.lock().unwrap().as_ref().map(|(at,c,reader_before,jvm_before,reader_after,jvm_after)|json!({"set_after_start_micros":at.duration_since(started).as_micros(),"return_latency_micros":returned.saturating_duration_since(*at).as_micros(),"set_before_return":*at<=returned,"work_after_set":counters(*c),"actual_body_starts_after_set_bounds":{"lower":jarde_reader::observation::snapshot().body_decode_attempts-reader_after.body_decode_attempts,"upper":jarde_reader::observation::snapshot().body_decode_attempts-reader_before.body_decode_attempts},"jvm_work_after_set_lower":jvm_after.since(jarde_jvm::observation::snapshot()),"jvm_work_after_set_upper":jvm_before.since(jarde_jvm::observation::snapshot())}));
        text_bytes=sink.max_text_bytes; hash=sink.hash;
        extra=json!({"report":report.map(|x|value(&x)).unwrap_or_else(|e|json!({"error":e.to_string()})),"events_prefix":sink.events,"events_prefix_cap":256,"offered_methods":sink.methods,"missing_prepared":sink.missing_prepared,"max_offered_result_weight":sink.max_owned_weight,"max_offered_text_bytes":sink.max_text_bytes,"encoded_buffer_peak_bytes":sink.encoded_peak,"encoded_total_bytes":sink.encoded_total,"retained_consumer_products":0,"token":token_data,"workers_alive_after_return":watch.alive(),"workers_high_water":watch.high_water(),"probe":probe.reading()});
    }
    let elapsed = started.elapsed().as_micros();
    let reader_after = jarde_reader::observation::snapshot();
    let reader_a = value(&reader_before); let reader_z = value(&reader_after);
    let mut work = serde_json::Map::new();
    for (k,v) in reader_z.as_object().unwrap() {
        if !k.ends_with("_live") && !k.ends_with("_peak") {
            work.insert(k.clone(),json!(v.as_u64().unwrap()-reader_a[k].as_u64().unwrap()));
        }
    }
    json!({"observed_work":{"reader":work,"jvm":jvm_before.since(jarde_jvm::observation::snapshot())},
        "retention":{"reader_begin":reader_before,"reader_end":reader_after,"java_begin":java_before,"java_end":jarde_java::observation::snapshot()},
        "micros":elapsed,"counts":counters(before),"usage":b.usage(),"cache":cache.as_ref().map(|x|value(&x.report())),"text_bytes":text_bytes,"semantic_text_digest":hash.finalize().to_hex().as_str(),"details":extra})
}

#[test]
#[ignore = "S0 measurement only"]
fn measure_s0() {
    let cfg:Value=serde_json::from_str(&std::env::var("JARDE_S0_CONFIG").unwrap()).unwrap();
    let limits:Limits=serde_json::from_value(cfg["limits"].clone()).unwrap();
    let mode=cfg["cache"].as_str().unwrap();
    let cache=match mode { "disabled"=>None,"tiny"=>Some(FactsCache::current(FactsCapacity::new(1,4096))),_=>Some(FactsCache::current(FactsCapacity::new(16384,1<<27))) };
    let engine=Engine::new();let mut b=budget(&limits,&cache);
    let before_open=jarde_reader::observation::snapshot();
    let clock=Instant::now();let snapshot=engine.open(ArtifactInput::Path(cfg["input"].as_str().unwrap().into()),&mut b).unwrap();
    let open_micros=clock.elapsed().as_micros();
    let open_after=jarde_reader::observation::snapshot();
    let open_work=json!({"file_read_attempts":open_after.file_read_attempts-before_open.file_read_attempts,"file_read_bytes":open_after.file_read_bytes-before_open.file_read_bytes,"file_read_eofs":open_after.file_read_eofs-before_open.file_read_eofs,"file_read_errors":open_after.file_read_errors-before_open.file_read_errors});
    let mut env:EnvironmentRequest=serde_json::from_value(cfg["environment"].clone()).unwrap();env.snapshot=snapshot.id().clone();
    let evidence=if cfg["evidence"]=="all" {RecoveryEvidenceRequest::all()} else {RecoveryEvidenceRequest::essential()};
    let warm=if mode=="hot" {Some(operation(&cfg,&engine,&snapshot,&env,&cache,&limits,&evidence))}else{None};
    // Censored WAR bulk runs keep periodic, explicitly non-final observation prefixes.
    let (stop, stopped)=mpsc::channel::<()>();
    let progress = if cfg["label"]=="s2-009" && cfg["shape"]=="bulk" {
        Some(std::thread::spawn(move || {
            use std::io::Write;
            let began=Instant::now();
            while stopped.recv_timeout(Duration::from_millis(250))==Err(mpsc::RecvTimeoutError::Timeout) {
                println!("S0_OBSERVATION {}",json!({"micros":began.elapsed().as_micros(),"reader":jarde_reader::observation::snapshot(),"jvm":jarde_jvm::observation::snapshot(),"java":jarde_java::observation::snapshot(),"final":false}));
                std::io::stdout().flush().unwrap();
            }
        }))
    } else { None };
    let sample=operation(&cfg,&engine,&snapshot,&env,&cache,&limits,&evidence);
    drop(stop);
    if let Some(progress)=progress { progress.join().unwrap(); }
    println!("{}",json!({"config":cfg,"snapshot":snapshot.id(),"open_micros":open_micros,"open_actual_reads":open_work,"open_usage":b.usage(),"warmup":warm,"sample":sample,"observability":{"units":"artifact file read calls/returned bytes and entry wrapped-reader calls/output bytes (Deflate output); JVM pipeline and phase calls/publications; facts/handles, Java AST nodes and declared class candidate payload live/peak object counts, not heap bytes","scope":"artifact open file reads are reported separately from request entry reads; slice parsing does not make stream calls; JVM phase order pipeline/raw_cfg/contexts/canonical/frames/ssa; Java instrumented payload types are listed in observation module; uninstrumented allocation bytes remain process RSS only","reset":"peaks reset at quiescent operation start, including live warm cache objects; work is a monotonic delta"}}));
}
