//! P4 1.2 acceptance: the modern structural facts — record components, the deferred
//! constant-dynamic graph, modern string-concat sites — and the release-bound registry's answer
//! about the attributes they are read from.
//!
//! Two kinds of input, for the two kinds of claim this slice makes:
//!
//! * **Real compiled samples** under `fixtures/p4-modern/` (`javac 23.0.1`; see that directory's
//!   `README.md` for commands, byte counts and SHA-256). They pin what a compiler really emits:
//!   a `Record` attribute at major 60, `PermittedSubclasses` at 61, `NestHost`/`NestMembers`,
//!   a `Module` attribute, two `StringConcatFactory` sites, and a Java 8 class whose concat is a
//!   `StringBuilder` chain. The release rules are exercised by patching only the two version fields
//!   of those same files, the convention P4 1.1's `p4_feature_registry.rs` already uses.
//! * **One hand-built class** (`condy_fixture`), because `javac 23.0.1` emits no
//!   `CONSTANT_Dynamic` at all (recorded in the fixture README): the A05 graph needs exact
//!   constant-pool indexes, bootstrap indexes and argument positions, and a compiled sample cannot
//!   be asked to place them. P1's `p1_xref_bootstrap.rs` builds its condy fixtures the same way.
//!
//! The claims these tests make about the graph are about *structure*: a shared node is expanded
//! once, a cycle is recorded instead of followed, the edge and depth budgets stop the walk without
//! failing the read, and no bootstrap is executed — the fixture's bootstrap handle names a member
//! no class file here declares.

use jarde::{
    AttributePlacement, Budget, ClassfileLocation, ConcatStrategy, CondyEdgeKind, CondyNodeKind,
    CondyNodeRef, CondyStop, ConstantPoolTagStatus, InspectionMode, Limits, MethodSelector,
    ModernFeature, ModernOrigin, OutputLevel, OutputLevelStatus, attribute_facts, class_facts,
    cp_utf8, feature_registry, inspect_header, inspect_method_bytecode, modern_facts,
};
const RECORD_FIXTURE: &[u8] = include_bytes!("fixtures/p4-modern/v16/RecordSample.class");
const SEALED_FIXTURE: &[u8] = include_bytes!("fixtures/p4-modern/v17/SealedSample.class");
const NEST_FIXTURE: &[u8] = include_bytes!("fixtures/p4-modern/v17/NestSample.class");
const MODULE_FIXTURE: &[u8] = include_bytes!("fixtures/p4-modern/v17/module-info.class");
const CONCAT_FIXTURE: &[u8] = include_bytes!("fixtures/p4-modern/v17/ConcatSample.class");
const JAVA8_FIXTURE: &[u8] = include_bytes!("fixtures/p4-modern/v8/ConcatJava8.class");

fn limits() -> Limits {
    Limits {
        input_bytes: u64::MAX,
        archive_entries: u64::MAX,
        entry_bytes: u64::MAX,
        read_bytes: u64::MAX,
        class_bytes: u64::MAX,
        attribute_bytes: u64::MAX,
        code_bytes: u64::MAX,
        result_items: u64::MAX,
        output_bytes: u64::MAX,
        class_headers: u64::MAX,
        method_bodies: u64::MAX,
        ir_items: u64::MAX,
        ir_edges: u64::MAX,
        analysis_steps: u64::MAX,
        normalization_clones: u64::MAX,
        nested_depth: u64::MAX,
        dependency_depth: u64::MAX,
        elapsed_millis: u64::MAX,
    }
}

/// Reads one class's modern facts at `level` and returns them with the bytes they were read from.
fn read(
    bytes: &[u8],
    level: OutputLevel,
    limits: Limits,
) -> (jarde::ModernFacts, jarde::UsageSnapshot) {
    let mut budget = Budget::new(limits);
    let facts = class_facts(bytes, &mut budget).expect("the fixture's class structure is readable");
    let modern =
        modern_facts(bytes, &facts, level, &mut budget).expect("modern facts are readable");
    (modern, budget.usage().clone())
}

/// The same class with only its `minor_version`/`major_version` fields patched.
fn with_version(bytes: &[u8], major: u16) -> Vec<u8> {
    let mut patched = bytes.to_vec();
    patched[4..6].copy_from_slice(&0u16.to_be_bytes());
    patched[6..8].copy_from_slice(&major.to_be_bytes());
    patched
}

/// The first reach of `node` that is not a repeat: the path that expanded it.
fn unshared_reach(site: &jarde::CondyUseSite, node: CondyNodeRef) -> &jarde::CondyReach {
    site.reaches
        .iter()
        .find(|reach| reach.node == node && reach.repeat.is_none())
        .unwrap_or_else(|| panic!("{node:?} is reached from {:?}", site.site))
}

fn codes(facts: &jarde::ModernFacts) -> Vec<&str> {
    facts
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.as_str())
        .collect()
}

// ---------------------------------------------------------------------------
// The hand-built constant-dynamic fixture
// ---------------------------------------------------------------------------

fn u16b(buffer: &mut Vec<u8>, value: u16) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

fn u32b(buffer: &mut Vec<u8>, value: u32) {
    buffer.extend_from_slice(&value.to_be_bytes());
}

/// Constant-pool builder: appends entries in order and reports their 1-based indexes.
#[derive(Default)]
struct Pool {
    entries: Vec<Vec<u8>>,
}

impl Pool {
    fn push(&mut self, entry: Vec<u8>) -> u16 {
        self.entries.push(entry);
        u16::try_from(self.entries.len()).expect("the fixture's constant pool fits u16")
    }

    fn utf8(&mut self, text: &[u8]) -> u16 {
        let mut entry = vec![1];
        u16b(
            &mut entry,
            u16::try_from(text.len()).expect("fixture text fits u16"),
        );
        entry.extend_from_slice(text);
        self.push(entry)
    }

    fn class(&mut self, name: u16) -> u16 {
        let mut entry = vec![7];
        u16b(&mut entry, name);
        self.push(entry)
    }

    fn string(&mut self, value: u16) -> u16 {
        let mut entry = vec![8];
        u16b(&mut entry, value);
        self.push(entry)
    }

    fn name_and_type(&mut self, name: u16, descriptor: u16) -> u16 {
        let mut entry = vec![12];
        u16b(&mut entry, name);
        u16b(&mut entry, descriptor);
        self.push(entry)
    }

    fn method_ref(&mut self, class: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![10];
        u16b(&mut entry, class);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    fn method_handle(&mut self, kind: u8, reference: u16) -> u16 {
        let mut entry = vec![15, kind];
        u16b(&mut entry, reference);
        self.push(entry)
    }

    /// `CONSTANT_Dynamic` (17): the tag the A05 graph is about.
    fn dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![17];
        u16b(&mut entry, bootstrap);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    fn invoke_dynamic(&mut self, bootstrap: u16, name_and_type: u16) -> u16 {
        let mut entry = vec![18];
        u16b(&mut entry, bootstrap);
        u16b(&mut entry, name_and_type);
        self.push(entry)
    }

    fn declared(&self) -> u16 {
        u16::try_from(self.entries.len() + 1).expect("the fixture's constant pool fits u16")
    }

    fn bytes(&self) -> Vec<u8> {
        self.entries.iter().flatten().copied().collect()
    }
}

struct MethodSpec {
    access: u16,
    name: u16,
    descriptor: u16,
    attributes: Vec<(u16, Vec<u8>)>,
}

struct AttributeSpec {
    name: u16,
    content: Vec<u8>,
}

/// Assembles a class file around a pool and the declarations the caller makes.
fn class_bytes(
    pool: &Pool,
    major: u16,
    this_class: u16,
    super_class: u16,
    methods: &[MethodSpec],
    attributes: &[AttributeSpec],
) -> Vec<u8> {
    let mut bytes = 0xcafe_babe_u32.to_be_bytes().to_vec();
    u16b(&mut bytes, 0); // minor
    u16b(&mut bytes, major);
    u16b(&mut bytes, pool.declared());
    bytes.extend_from_slice(&pool.bytes());
    u16b(&mut bytes, 0x0021); // ACC_PUBLIC | ACC_SUPER
    u16b(&mut bytes, this_class);
    u16b(&mut bytes, super_class);
    u16b(&mut bytes, 0); // interfaces
    u16b(&mut bytes, 0); // fields
    u16b(
        &mut bytes,
        u16::try_from(methods.len()).expect("fixture methods fit u16"),
    );
    for method in methods {
        u16b(&mut bytes, method.access);
        u16b(&mut bytes, method.name);
        u16b(&mut bytes, method.descriptor);
        u16b(
            &mut bytes,
            u16::try_from(method.attributes.len()).expect("fixture attributes fit u16"),
        );
        for (name, content) in &method.attributes {
            u16b(&mut bytes, *name);
            u32b(
                &mut bytes,
                u32::try_from(content.len()).expect("attribute content fits u32"),
            );
            bytes.extend_from_slice(content);
        }
    }
    u16b(
        &mut bytes,
        u16::try_from(attributes.len()).expect("fixture attributes fit u16"),
    );
    for attribute in attributes {
        u16b(&mut bytes, attribute.name);
        u32b(
            &mut bytes,
            u32::try_from(attribute.content.len()).expect("attribute content fits u32"),
        );
        bytes.extend_from_slice(&attribute.content);
    }
    bytes
}

/// A `Code` body with no exception table and no nested attribute.
fn code_body(instructions: &[u8]) -> Vec<u8> {
    let mut body = Vec::new();
    u16b(&mut body, 1); // max_stack
    u16b(&mut body, 0); // max_locals
    u32b(
        &mut body,
        u32::try_from(instructions.len()).expect("fixture code fits u32"),
    );
    body.extend_from_slice(instructions);
    u16b(&mut body, 0);
    u16b(&mut body, 0);
    body
}

/// A `BootstrapMethods` body: `(handle, arguments)` per entry.
fn bootstrap_body(entries: &[(u16, Vec<u16>)]) -> Vec<u8> {
    let mut body = Vec::new();
    u16b(
        &mut body,
        u16::try_from(entries.len()).expect("fixture bootstrap entries fit u16"),
    );
    for (handle, arguments) in entries {
        u16b(&mut body, *handle);
        u16b(
            &mut body,
            u16::try_from(arguments.len()).expect("fixture arguments fit u16"),
        );
        for argument in arguments {
            u16b(&mut body, *argument);
        }
    }
    body
}

/// Constant-pool indexes the condy fixture's tests assert against.
struct CondyIndexes {
    condy1: u16,
    condy2: u16,
    condy3: u16,
    condy4: u16,
    condy5: u16,
    handle: u16,
    leaf: u16,
    bootstrap_name: u16,
}

/// The A05 shape: one shared subgraph two entry points reach, one two-node cycle, and a bootstrap
/// handle that names a member no class file declares.
///
/// Bootstrap table:
///
/// ```text
/// 0: (handle, [condy2])   condy1 -> condy2
/// 1: (handle, [condy3])   condy2 -> condy3        the node entry points 1 and 2 share
/// 2: (handle, [leaf])     condy3 -> "leaf"        shared with the path through condy1
/// 3: (handle, [condy5])   condy4 -> condy5
/// 4: (handle, [condy4])   condy5 -> condy4        the cycle
/// ```
///
/// The handle names `p/NoSuchFactory.boom` (JVMS 4.7.23's `bootstrap_method_ref`): a reader that
/// resolved or executed the bootstrap could not report this class's facts at all.
fn condy_fixture() -> (Vec<u8>, CondyIndexes) {
    let mut pool = Pool::default();
    let this_name = pool.utf8(b"p/Condy");
    let this_class = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let method_name = pool.utf8(b"use");
    let method_descriptor = pool.utf8(b"()V");
    let code_name = pool.utf8(b"Code");
    let name1 = pool.utf8(b"condy1");
    let name2 = pool.utf8(b"condy2");
    let name3 = pool.utf8(b"condy3");
    let name4 = pool.utf8(b"condy4");
    let name5 = pool.utf8(b"condy5");
    let descriptor = pool.utf8(b"Ljava/lang/Object;");
    let nat1 = pool.name_and_type(name1, descriptor);
    let nat2 = pool.name_and_type(name2, descriptor);
    let nat3 = pool.name_and_type(name3, descriptor);
    let nat4 = pool.name_and_type(name4, descriptor);
    let nat5 = pool.name_and_type(name5, descriptor);
    let factory_name = pool.utf8(b"p/NoSuchFactory");
    let factory_class = pool.class(factory_name);
    let boom = pool.utf8(b"boom");
    let boom_descriptor = pool.utf8(
        b"(Ljava/lang/invoke/MethodHandles$Lookup;Ljava/lang/String;Ljava/lang/invoke/MethodType;)Ljava/lang/invoke/CallSite;",
    );
    let boom_nat = pool.name_and_type(boom, boom_descriptor);
    let boom_ref = pool.method_ref(factory_class, boom_nat);
    let handle = pool.method_handle(6, boom_ref);
    let leaf_text = pool.utf8(b"leaf");
    let leaf = pool.string(leaf_text);
    let condy1 = pool.dynamic(0, nat1);
    let condy2 = pool.dynamic(1, nat2);
    let condy3 = pool.dynamic(2, nat3);
    let condy4 = pool.dynamic(3, nat4);
    let condy5 = pool.dynamic(4, nat5);
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let bootstrap = bootstrap_body(&[
        (handle, vec![condy2]),
        (handle, vec![condy3]),
        (handle, vec![leaf]),
        (handle, vec![condy5]),
        (handle, vec![condy4]),
    ]);
    // `ldc_w` on the first condy, `pop`, `return`: one entry point is really used by a body.
    let mut instructions = vec![0x13];
    u16b(&mut instructions, condy1);
    instructions.push(0x57);
    instructions.push(0xb1);
    let bytes = class_bytes(
        &pool,
        55,
        this_class,
        object_class,
        &[MethodSpec {
            access: 0x0009,
            name: method_name,
            descriptor: method_descriptor,
            attributes: vec![(code_name, code_body(&instructions))],
        }],
        &[AttributeSpec {
            name: bootstrap_name,
            content: bootstrap,
        }],
    );
    (
        bytes,
        CondyIndexes {
            condy1,
            condy2,
            condy3,
            condy4,
            condy5,
            handle,
            leaf,
            bootstrap_name,
        },
    )
}

// ---------------------------------------------------------------------------
// A05: the shared graph, the cycle and the budgets
// ---------------------------------------------------------------------------

#[test]
fn a_shared_subgraph_is_expanded_once_and_each_entry_point_keeps_its_own_path() {
    let (bytes, index) = condy_fixture();
    let (facts, _) = read(&bytes, OutputLevel::Java8, limits());
    let graph = &facts.condy;

    assert!(graph.is_complete(), "the fixture fits every budget");
    assert_eq!(graph.bootstrap_entries, 5);
    match graph.dynamic_tag {
        ConstantPoolTagStatus::Registered { rule } => {
            assert_eq!(rule.tag, 17);
            assert_eq!(rule.since, 55);
        }
        other => panic!("the registry registers CONSTANT_Dynamic from major 55 on: {other:?}"),
    }
    assert_eq!(
        graph.use_sites.len(),
        5,
        "every Dynamic entry is an entry point, in constant-pool order"
    );
    assert_eq!(
        graph
            .use_sites
            .iter()
            .map(|site| site.site)
            .collect::<Vec<_>>(),
        vec![
            CondyNodeRef::ConstantPool {
                index: index.condy1
            },
            CondyNodeRef::ConstantPool {
                index: index.condy2
            },
            CondyNodeRef::ConstantPool {
                index: index.condy3
            },
            CondyNodeRef::ConstantPool {
                index: index.condy4
            },
            CondyNodeRef::ConstantPool {
                index: index.condy5
            },
        ]
    );

    // The graph itself: 5 dynamic entries, the method handle, the leaf string and the five table
    // entries — each node once, however many paths reach it.
    assert_eq!(graph.nodes.len(), 12, "{:#?}", graph.nodes);
    assert_eq!(graph.edges.len(), 15, "{:#?}", graph.edges);
    assert_eq!(
        graph.budget.nodes, 12,
        "IrItems counted the nodes of the graph: five dynamic entries, the handle, the leaf and the five table entries"
    );
    assert_eq!(
        graph.budget.edges, 15,
        "IrEdges counted the edges of the graph"
    );
    assert_eq!(graph.budget.steps, 35, "AnalysisSteps counted every visit");
    assert_eq!(
        graph.budget.max_depth, 6,
        "the deepest path is condy1 -> its table entry -> condy2 -> its table entry -> condy3 -> its table entry -> leaf"
    );
    assert_eq!(graph.budget.stopped, None);
    assert_eq!(graph.cycles.len(), 2, "one cycle per entry point of it");
    let table = facts
        .attributes
        .iter()
        .find(|row| row.name.0 == b"BootstrapMethods")
        .expect("the table's own attribute is placed");
    assert!(table.read, "the pass reads the table it walks");
    assert_eq!(table.name_index, index.bootstrap_name);
    assert_eq!(
        cp_utf8(
            &class_facts(&bytes, &mut Budget::new(limits()))
                .unwrap()
                .constant_pool,
            table.name_index
        )
        .unwrap()
        .0,
        b"BootstrapMethods".to_vec(),
        "the index read back from the entry's own span spells the name the shell reports"
    );

    // The shared node is one node, and both entry points reach it with their own path.
    let condy3 = CondyNodeRef::ConstantPool {
        index: index.condy3,
    };
    let shared_bsm = CondyNodeRef::Bootstrap { index: 2 };
    assert_eq!(
        graph.node(condy3).map(|node| node.kind),
        Some(Some(CondyNodeKind::Dynamic))
    );
    assert_eq!(
        graph.node(shared_bsm).map(|node| node.kind),
        Some(None),
        "a BootstrapMethods entry is a table position, not a constant-pool entry"
    );
    let first = &graph.use_sites[0];
    let second = &graph.use_sites[1];
    let first_leaf = unshared_reach(first, CondyNodeRef::ConstantPool { index: index.leaf });
    let second_leaf = unshared_reach(second, CondyNodeRef::ConstantPool { index: index.leaf });
    assert_eq!(first_leaf.depth(), 6);
    assert_eq!(second_leaf.depth(), 4);
    // From the shared node onwards both entry points walk the *same* edges, because the expansion is
    // stored once: an edge two paths share keeps one ordinal.
    let first_from_shared = &first_leaf.via[unshared_reach(first, condy3).depth()..];
    let second_from_shared = &second_leaf.via[unshared_reach(second, condy3).depth()..];
    assert_eq!(
        first_from_shared, second_from_shared,
        "the subgraph both entry points walk is stored once"
    );
    assert_eq!(
        first_from_shared.len(),
        2,
        "the shared node's own bootstrap entry and that entry's reference to the leaf"
    );
    // Every element of a path is a real edge, and the path really starts at the entry point.
    for (site, leaf) in [(first, first_leaf), (second, second_leaf)] {
        assert_eq!(
            graph.edges[usize::try_from(leaf.via[0]).unwrap()].from,
            site.site
        );
        assert_eq!(
            graph.edges[usize::try_from(leaf.via[0]).unwrap()].kind,
            CondyEdgeKind::Bootstrap
        );
        let last = graph.edges[usize::try_from(*leaf.via.last().unwrap()).unwrap()].to;
        assert_eq!(last, leaf.node, "the last edge of a path reaches its node");
    }
    // A repeat is a path, not an expansion: the handle is reached twice from the first entry point
    // and the walk stops at it both times.
    assert_eq!(
        first
            .reaches
            .iter()
            .filter(|reach| reach.node
                == CondyNodeRef::ConstantPool {
                    index: index.handle
                })
            .count(),
        3,
        "{:#?}",
        first.reaches
    );
    assert_eq!(
        first.reaches.last().map(|reach| reach.node),
        Some(CondyNodeRef::ConstantPool { index: index.leaf })
    );
}

#[test]
fn a_cycle_is_recorded_as_facts_instead_of_being_followed() {
    let (bytes, index) = condy_fixture();
    let (facts, _) = read(&bytes, OutputLevel::Java8, limits());
    let graph = &facts.condy;
    let condy4 = CondyNodeRef::ConstantPool {
        index: index.condy4,
    };
    let condy5 = CondyNodeRef::ConstantPool {
        index: index.condy5,
    };

    assert_eq!(
        graph
            .cycles
            .iter()
            .map(|cycle| (cycle.site, cycle.node))
            .collect::<Vec<_>>(),
        vec![(3, condy4), (4, condy5)],
        "condy4 -> condy5 -> condy4 is one cycle, and condy5 -> condy4 -> condy5 is the same cycle \
         entered from the other node"
    );
    for cycle in &graph.cycles {
        let site = &graph.use_sites[cycle.site];
        let reach = &site.reaches[usize::try_from(cycle.reach).unwrap()];
        assert_eq!(reach.node, cycle.node);
        assert_eq!(
            reach.repeat,
            Some(0),
            "the repeat names the entry point's own reach, which is what makes it a cycle"
        );
        assert_eq!(reach.depth(), 4);
        let last = &graph.edges[usize::try_from(*reach.via.last().unwrap()).unwrap()];
        assert_eq!(
            last.to, site.site,
            "the path's last edge closes on the node the walk started from"
        );
        assert_eq!(last.kind, CondyEdgeKind::BootstrapArgument);
    }
    // The walk terminated: every entry point is in the answer, and the cycle's nodes were expanded
    // once, not once per turn around the cycle.
    assert_eq!(graph.use_sites.len(), 5);
    assert_eq!(
        graph
            .nodes
            .iter()
            .filter(|node| node.node == condy4 || node.node == condy5)
            .count(),
        2
    );
    assert!(graph.is_complete());
}

#[test]
fn the_edge_budget_stops_the_walk_and_the_read_still_reports_its_prefix() {
    let (bytes, _) = condy_fixture();
    let limited = Limits {
        ir_edges: 3,
        ..limits()
    };
    let (facts, usage) = read(&bytes, OutputLevel::Java8, limited);
    let graph = &facts.condy;

    assert_eq!(graph.budget.edges, 3);
    assert_eq!(graph.budget.stopped, Some(CondyStop::Edges { limit: 3 }));
    assert_eq!(
        usage.ir_edges, 3,
        "the budget the caller set is the budget the request consumed"
    );
    assert!(
        !graph.is_complete(),
        "a truncated graph is not a complete one"
    );
    assert_eq!(
        graph.edges.len(),
        3,
        "the discovered prefix is published, not discarded"
    );
    assert_eq!(
        graph.use_sites.len(),
        1,
        "the entry point the walk was inside keeps the reaches it recorded"
    );
    let site = &graph.use_sites[0];
    assert_eq!(site.reaches.len(), 4, "{:#?}", site.reaches);
    assert_eq!(
        site.reaches
            .iter()
            .filter(|reach| reach.repeat.is_none())
            .count(),
        4
    );
    assert!(graph.cycles.is_empty());
    assert!(graph.nodes.len() < 12);
}

#[test]
fn the_depth_budget_stops_the_walk_at_the_level_it_names() {
    let (bytes, _) = condy_fixture();
    let limited = Limits {
        dependency_depth: 2,
        ..limits()
    };
    let (facts, usage) = read(&bytes, OutputLevel::Java8, limited);
    let graph = &facts.condy;

    assert_eq!(
        graph.budget.stopped,
        Some(CondyStop::Depth {
            limit: 2,
            entering: 3
        })
    );
    assert_eq!(graph.budget.max_depth, 2);
    assert_eq!(
        usage.dependency_depth, 2,
        "the high-water mark is the walk's"
    );
    assert!(!graph.is_complete());
    assert_eq!(graph.use_sites.len(), 1);
    assert_eq!(
        graph.use_sites[0].reaches.len(),
        4,
        "{:#?}",
        graph.use_sites[0].reaches
    );
    assert!(
        graph
            .use_sites
            .iter()
            .flat_map(|site| &site.reaches)
            .all(|reach| reach.depth() <= 2),
        "no path in a depth-limited answer exceeds the limit"
    );
}

#[test]
fn the_node_and_step_budgets_stop_the_walk_at_the_dimension_they_name() {
    let (bytes, _) = condy_fixture();
    // One node is the entry point itself: the second one — the table entry its first edge reaches —
    // is refused, and the walk stops with the stop it names.
    let (facts, usage) = read(
        &bytes,
        OutputLevel::Java8,
        Limits {
            ir_items: 1,
            ..limits()
        },
    );
    assert_eq!(
        facts.condy.budget.stopped,
        Some(CondyStop::Nodes { limit: 1 })
    );
    assert_eq!(facts.condy.budget.nodes, 1);
    assert_eq!(usage.ir_items, 1);
    assert!(facts.condy.edges.is_empty());
    assert_eq!(facts.condy.use_sites.len(), 1);
    assert_eq!(facts.condy.use_sites[0].reaches.len(), 1);
    assert!(!facts.is_complete());

    // One visit is the entry point's own: the walk stops before it can enter the node its edge
    // reaches.
    let (facts, usage) = read(
        &bytes,
        OutputLevel::Java8,
        Limits {
            analysis_steps: 1,
            ..limits()
        },
    );
    assert_eq!(
        facts.condy.budget.stopped,
        Some(CondyStop::Steps { limit: 1 })
    );
    assert_eq!(facts.condy.budget.steps, 1);
    assert_eq!(usage.analysis_steps, 1);
    assert_eq!(facts.condy.use_sites.len(), 1);
    assert_eq!(facts.condy.use_sites[0].reaches.len(), 1);
    assert!(!facts.is_complete());
}

#[test]
fn a_bootstrap_handle_that_names_nothing_is_never_resolved() {
    let (bytes, index) = condy_fixture();
    // The fixture's handle is `REF_invokeStatic p/NoSuchFactory.boom:(...)CallSite` — an owner no
    // class file in this repository declares. Nothing about that reference is resolved, loaded or
    // called: the graph is published, and the handle is a node with a constant-pool origin.
    let (facts, _) = read(&bytes, OutputLevel::Java8, limits());
    let graph = &facts.condy;
    let handle = CondyNodeRef::ConstantPool {
        index: index.handle,
    };
    let node = graph.node(handle).expect("the handle is a node");
    assert_eq!(node.kind, Some(CondyNodeKind::MethodHandle));
    assert!(
        node.span.is_some(),
        "the node keeps its own class-file range"
    );
    assert_eq!(
        graph.edges.iter().filter(|edge| edge.to == handle).count(),
        5,
        "every bootstrap entry names the same handle, and only its reference is recorded"
    );
    assert!(
        graph
            .edges
            .iter()
            .all(|edge| edge.kind != CondyEdgeKind::BootstrapHandle || edge.to == handle),
        "the walk stops at the handle: it never follows the member the handle names"
    );
    assert!(graph.is_complete());
    assert!(facts.diagnostics.is_empty(), "{:?}", facts.diagnostics);
}

#[test]
fn a_dynamic_entry_naming_an_undeclared_bootstrap_entry_is_diagnosed() {
    let mut pool = Pool::default();
    let this_name = pool.utf8(b"p/Dangling");
    let this_class = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let name = pool.utf8(b"condy");
    let descriptor = pool.utf8(b"Ljava/lang/Object;");
    let nat = pool.name_and_type(name, descriptor);
    let handle_name = pool.utf8(b"p/Custom");
    let handle_class = pool.class(handle_name);
    let bsm_name = pool.utf8(b"bsm");
    let bsm_descriptor = pool.utf8(b"()V");
    let bsm_nat = pool.name_and_type(bsm_name, bsm_descriptor);
    let bsm_ref = pool.method_ref(handle_class, bsm_nat);
    let handle = pool.method_handle(6, bsm_ref);
    // The entry names bootstrap 7; the attribute declares one entry.
    let condy = pool.dynamic(7, nat);
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let bytes = class_bytes(
        &pool,
        55,
        this_class,
        object_class,
        &[],
        &[AttributeSpec {
            name: bootstrap_name,
            content: bootstrap_body(&[(handle, vec![condy])]),
        }],
    );
    let (facts, _) = read(&bytes, OutputLevel::Java8, limits());
    assert_eq!(
        codes(&facts),
        vec!["classfile_condy_bootstrap_out_of_range"],
        "the dangling reference is diagnosed once, in constant-pool order"
    );
    let diagnostic = &facts.diagnostics[0];
    assert_eq!(
        diagnostic.message,
        "constant-pool entry 15 names BootstrapMethods entry 7, which the attribute does not declare (1 declared)"
    );
    assert!(
        diagnostic.provenance.is_none(),
        "a reader fact pass has no snapshot identity to name"
    );
    // The reference is still structure: the node exists, it is undeclared by the table, and it has
    // no outgoing edge.
    let graph = &facts.condy;
    assert_eq!(graph.bootstrap_entries, 1);
    let dangling = CondyNodeRef::Bootstrap { index: 7 };
    assert_eq!(graph.node(dangling).map(|node| node.kind), Some(None));
    assert_eq!(
        graph
            .edges
            .iter()
            .filter(|edge| edge.from == dangling)
            .count(),
        0
    );
    assert!(graph.is_complete());
}

// ---------------------------------------------------------------------------
// Record and sealed declarations, and the registry's placement of the attributes
// ---------------------------------------------------------------------------

#[test]
fn a_real_record_class_reports_components_with_their_origins() {
    let (facts, _) = read(RECORD_FIXTURE, OutputLevel::Java8, limits());
    let record = facts
        .record
        .as_ref()
        .expect("the registry places Record at major 60");
    assert_eq!(facts.major_version, 60);
    assert_eq!(record.rule.name, "Record");
    assert_eq!(record.rule.since, 60);
    assert_eq!(record.rule.source, "JVMS 4.7.30");
    assert_eq!(
        record.rule.locations,
        [ClassfileLocation::ClassFile].as_slice()
    );

    let names: Vec<&[u8]> = record
        .components
        .iter()
        .map(|component| component.name.0.as_slice())
        .collect();
    assert_eq!(names, vec![b"count".as_slice(), b"label", b"stamp"]);
    let descriptors: Vec<&[u8]> = record
        .components
        .iter()
        .map(|component| component.descriptor.0.as_slice())
        .collect();
    assert_eq!(
        descriptors,
        vec![b"I".as_slice(), b"Ljava/lang/String;", b"J"]
    );
    assert_eq!(
        record
            .components
            .iter()
            .map(|component| component.index)
            .collect::<Vec<_>>(),
        vec![0, 1, 2]
    );
    // The second component carries its own `attribute_info` list, kept as shells: the sample's
    // `@Marker` targets record components, so `javac` writes its `RuntimeVisibleAnnotations` inside
    // the `Record` attribute itself, and the reader reports the entry rather than interpreting it.
    let annotated = &record.components[1];
    assert!(
        !annotated.attributes.is_empty(),
        "the component's own attribute list is part of the fact"
    );
    assert!(
        annotated
            .attributes
            .iter()
            .any(|shell| shell.name.0 == b"RuntimeVisibleAnnotations"),
        "{:#?}",
        annotated.attributes
    );
    let mut budget = Budget::new(limits());
    let class = class_facts(RECORD_FIXTURE, &mut budget).unwrap();
    for shell in &annotated.attributes {
        // Every shell names its own index, and that index really spells the shell's name.
        assert_eq!(
            cp_utf8(&class.constant_pool, shell.name_index).unwrap(),
            shell.name
        );
        assert_eq!(shell.span.start + 6, shell.content_span.start);
        assert_eq!(shell.span.length, shell.content_span.length + 6);
        assert!(shell.span.start + shell.span.length <= RECORD_FIXTURE.len() as u64);
    }
    assert_eq!(record.span.start + 6, record.content_span.start);
    assert!(record.span.start + record.span.length <= RECORD_FIXTURE.len() as u64);
    assert!(
        record.components[0].attributes.is_empty() && record.components[2].attributes.is_empty(),
        "the reader reports the components that carry attributes and no others: {:?}",
        record
            .components
            .iter()
            .map(|component| component.attributes.len())
            .collect::<Vec<_>>()
    );

    // The placement row repeats the rule and the origin, and `javap` reports the same index.
    let row = facts
        .attributes
        .iter()
        .find(|row| row.name.0 == b"Record")
        .expect("the registered name is listed");
    assert_eq!(row.name_index, 40, "javap reports #40 = Utf8 Record");
    assert!(row.read);
    assert_eq!(
        row.placement,
        AttributePlacement::Legal { rule: record.rule }
    );
    assert_eq!(row.span, record.span);
}

#[test]
fn a_record_attribute_below_its_release_is_diagnosed_and_its_content_is_not_read() {
    let (legal, legal_usage) = read(RECORD_FIXTURE, OutputLevel::Java8, limits());
    let bytes = with_version(RECORD_FIXTURE, 59);
    let (early, early_usage) = read(&bytes, OutputLevel::Java8, limits());

    assert!(
        early.record.is_none(),
        "no release rule places Record at 59, so no component fact is published"
    );
    assert_eq!(
        codes(&early),
        vec!["classfile_attribute_version_not_applicable"]
    );
    assert_eq!(
        early.diagnostics[0].message,
        "attribute \"Record\" is registered from major 60 (JVMS 4.7.30); it is not valid at major 59"
    );
    assert!(early.diagnostics[0].provenance.is_none());
    let row = early
        .attributes
        .iter()
        .find(|row| row.name.0 == b"Record")
        .expect("the name is still placed, with the release's answer");
    assert!(
        matches!(
            row.placement,
            AttributePlacement::VersionNotApplicable { .. }
        ),
        "{:?}",
        row.placement
    );
    assert!(!row.read);
    assert_eq!(
        row.span,
        legal
            .attributes
            .iter()
            .find(|row| row.name.0 == b"Record")
            .unwrap()
            .span
    );
    assert!(
        early_usage.attribute_bytes < legal_usage.attribute_bytes,
        "the illegal entry's content was never read ({} < {})",
        early_usage.attribute_bytes,
        legal_usage.attribute_bytes
    );
}

#[test]
fn a_real_sealed_class_reports_its_permitted_subclasses() {
    let (facts, _) = read(SEALED_FIXTURE, OutputLevel::Java8, limits());
    let sealed = facts
        .permitted_subclasses
        .as_ref()
        .expect("the registry places PermittedSubclasses at major 61");
    assert_eq!(facts.major_version, 61);
    assert_eq!(sealed.rule.since, 61);
    assert!(
        sealed.rule.source.starts_with("JVMS 4.7."),
        "{:?}",
        sealed.rule
    );
    let permitted: Vec<&[u8]> = sealed
        .permitted
        .iter()
        .map(|name| name.0.as_slice())
        .collect();
    assert_eq!(permitted, vec![b"Alpha".as_slice(), b"Beta"]);
    assert_eq!(sealed.span.start + 6, sealed.content_span.start);
    let row = facts
        .attributes
        .iter()
        .find(|row| row.name.0 == b"PermittedSubclasses")
        .expect("the registered name is listed");
    assert!(row.read);
    assert_eq!(
        row.placement,
        AttributePlacement::Legal { rule: sealed.rule }
    );

    // One release earlier the same bytes say something different: the attribute is not placed.
    let (early, _) = read(
        &with_version(SEALED_FIXTURE, 60),
        OutputLevel::Java8,
        limits(),
    );
    assert!(early.permitted_subclasses.is_none());
    assert_eq!(
        codes(&early),
        vec!["classfile_attribute_version_not_applicable"]
    );
    assert!(early.diagnostics[0].message.contains("PermittedSubclasses"));
}

#[test]
fn the_placement_of_every_registered_attribute_names_its_release() {
    let registry = feature_registry();

    // A real nest: the host carries NestMembers, the member NestHost, both legal from major 55.
    let (facts, _) = read(NEST_FIXTURE, OutputLevel::Java8, limits());
    let members = facts
        .attributes
        .iter()
        .find(|row| row.name.0 == b"NestMembers")
        .expect("NestMembers is placed");
    let AttributePlacement::Legal { rule } = &members.placement else {
        panic!("NestMembers is legal at major 61: {:?}", members.placement);
    };
    assert_eq!(*rule, registry.attribute("NestMembers", 61).unwrap());
    assert_eq!(rule.since, 55);
    assert_eq!(rule.source, "JVMS 4.7.29");
    assert!(
        !members.read,
        "another reader pass owns the nestmate fact: this pass reports its placement and nothing else"
    );
    assert!(facts.diagnostics.is_empty());

    // The same bytes at major 54: the attribute is registered, but for a later release.
    let (early, _) = read(
        &with_version(NEST_FIXTURE, 54),
        OutputLevel::Java8,
        limits(),
    );
    assert_eq!(
        codes(&early),
        vec!["classfile_attribute_version_not_applicable"]
    );
    assert!(
        early
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.starts_with("attribute \"NestMembers\""))
    );

    // A real module: the attribute is legal from major 53, and the P1 fact reader still reads it.
    let (facts, _) = read(MODULE_FIXTURE, OutputLevel::Java8, limits());
    let module = facts
        .attributes
        .iter()
        .find(|row| row.name.0 == b"Module")
        .expect("Module is placed");
    let AttributePlacement::Legal { rule } = &module.placement else {
        panic!("Module is legal at major 61: {:?}", module.placement);
    };
    assert_eq!(rule.since, 53);
    assert!(!module.read);
    let mut budget = Budget::new(limits());
    let class = class_facts(MODULE_FIXTURE, &mut budget).unwrap();
    let module_facts = attribute_facts(
        MODULE_FIXTURE,
        &class.attributes,
        &class.constant_pool,
        &mut budget,
    )
    .unwrap()
    .module
    .expect("the P1 reader owns the Module structure");
    assert_eq!(
        module_facts.uses,
        vec![jarde::JvmBytes(b"java/util/spi/ToolProvider".to_vec())],
        "uses and provides are internal names, as the P1 fact reader publishes them"
    );
    assert_eq!(module_facts.provides.len(), 1);
    assert_eq!(
        module_facts.provides[0].service,
        jarde::JvmBytes(b"java/util/spi/ToolProvider".to_vec())
    );
    assert_eq!(
        module_facts.provides[0].implementations,
        vec![jarde::JvmBytes(b"p4/sample/Provider".to_vec())]
    );

    // A release the registry does not hold makes no claim about any name, and invents no diagnostic.
    let (future, _) = read(
        &with_version(RECORD_FIXTURE, 72),
        OutputLevel::Java8,
        limits(),
    );
    assert!(future.attributes.is_empty());
    assert!(future.record.is_none());
    assert!(future.diagnostics.is_empty(), "{:?}", future.diagnostics);
}

// ---------------------------------------------------------------------------
// Modern string concat
// ---------------------------------------------------------------------------

#[test]
fn a_real_concat_site_is_a_reader_fact_and_joins_to_the_bci_of_its_instruction() {
    let (facts, _) = read(CONCAT_FIXTURE, OutputLevel::Java8, limits());
    assert_eq!(facts.concat.len(), 2, "{:#?}", facts.concat);
    for site in &facts.concat {
        assert_eq!(site.strategy, ConcatStrategy::ConcatWithConstants);
        assert_eq!(
            site.factory_owner.0,
            b"java/lang/invoke/StringConcatFactory".to_vec()
        );
        assert_eq!(site.factory_name.0, b"makeConcatWithConstants".to_vec());
        assert_eq!(site.tag_rule.since, 51);
        let recipe = site.recipe.as_ref().expect("the strategy carries a recipe");
        assert!(
            recipe.0.contains(&1),
            "the recipe keeps javac's placeholder byte as it is: {:?}",
            recipe
        );
        assert_eq!(site.span.length, 5, "an InvokeDynamic entry is five bytes");
    }
    assert_eq!(
        facts
            .concat
            .iter()
            .map(|site| site.bootstrap_index)
            .collect::<Vec<_>>(),
        vec![0, 1],
        "each site names its own BootstrapMethods entry: javap reports the two entries too"
    );
    assert_eq!(
        facts
            .concat
            .iter()
            .map(|site| site.descriptor.0.as_slice())
            .collect::<Vec<_>>(),
        vec![
            b"(ILjava/lang/String;)Ljava/lang/String;".as_slice(),
            b"(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;"
        ]
    );

    // The BCI is the bytecode reader's fact, and the join key is the site's own constant-pool
    // index. The version fields are patched to 52 only because the strict bytecode reader validates
    // dialects up to that release; not one byte of the body or the constant pool changes.
    let patched = with_version(CONCAT_FIXTURE, 52);
    let bodies = [
        (
            b"mixed".as_slice(),
            b"(ILjava/lang/Object;)Ljava/lang/String;".as_slice(),
            &facts.concat[0],
            b"(ILjava/lang/String;)Ljava/lang/String;".as_slice(),
            5u32,
        ),
        (
            b"pair".as_slice(),
            b"(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;".as_slice(),
            &facts.concat[1],
            b"(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;".as_slice(),
            2u32,
        ),
    ];
    for (name, descriptor, site, site_descriptor, bci) in bodies {
        assert_eq!(site.descriptor.0.as_slice(), site_descriptor);
        let mut budget = Budget::new(limits());
        let body = inspect_method_bytecode(
            &patched,
            MethodSelector {
                name: jarde::JvmBytes(name.to_vec()),
                descriptor: jarde::JvmBytes(descriptor.to_vec()),
            },
            &mut budget,
        )
        .unwrap();
        let instruction = body
            .instructions
            .iter()
            .find(|instruction| instruction.constant_pool_index == Some(site.constant_pool_index))
            .expect("the site's entry is the entry the instruction consumed");
        assert_eq!(instruction.opcode, 0xba, "invokedynamic");
        assert_eq!(
            instruction.bci,
            bci,
            "javap reports the same offset for {}: {body:#?}",
            String::from_utf8_lossy(name)
        );
    }
}

#[test]
fn a_class_whose_invokedynamic_sites_are_not_concat_reports_no_concat_fact() {
    // `RecordSample` is full of indy sites: three of them, all bootstrapped by
    // `java/lang/runtime/ObjectMethods.bootstrap`. None is a concat site, and claiming one from the
    // instruction's shape would be exactly the name-based guess this pass must not make.
    let (facts, _) = read(RECORD_FIXTURE, OutputLevel::Java8, limits());
    assert!(facts.concat.is_empty(), "{:#?}", facts.concat);
    let mut budget = Budget::new(limits());
    let class = class_facts(RECORD_FIXTURE, &mut budget).unwrap();
    let indy = class
        .constant_pool
        .iter()
        .filter(|entry| matches!(entry.kind, jarde::CpEntryKind::InvokeDynamic { .. }))
        .count();
    assert_eq!(indy, 3);
    // The graph still opens on those sites: every dynamic entry is an entry point.
    assert_eq!(facts.condy.use_sites.len(), 3);
    assert_eq!(facts.condy.bootstrap_entries, 1);
    assert!(facts.condy.is_complete());

    // A Java 8 class has one indy site too — the lambda's metafactory — and still no concat fact.
    let (flat, _) = read(JAVA8_FIXTURE, OutputLevel::Java8, limits());
    assert!(flat.concat.is_empty());
    assert_eq!(flat.condy.use_sites.len(), 1);
    assert!(flat.is_complete());
}

#[test]
fn an_indy_site_whose_handle_is_not_a_member_gets_no_concat_fact() {
    let mut pool = Pool::default();
    let this_name = pool.utf8(b"p/Odd");
    let this_class = pool.class(this_name);
    let object_name = pool.utf8(b"java/lang/Object");
    let object_class = pool.class(object_name);
    let not_a_member = pool.utf8(b"not-a-member");
    // A `CONSTANT_MethodHandle` whose `reference_index` names a `CONSTANT_Utf8`: the site's factory
    // cannot be named, and the reader says so instead of guessing one.
    let handle = pool.method_handle(6, not_a_member);
    let name = pool.utf8(b"makeConcatWithConstants");
    let descriptor = pool.utf8(b"(Ljava/lang/String;)Ljava/lang/String;");
    let nat = pool.name_and_type(name, descriptor);
    let _site = pool.invoke_dynamic(0, nat);
    let bootstrap_name = pool.utf8(b"BootstrapMethods");
    let bytes = class_bytes(
        &pool,
        55,
        this_class,
        object_class,
        &[],
        &[AttributeSpec {
            name: bootstrap_name,
            content: bootstrap_body(&[(handle, vec![])]),
        }],
    );
    let (facts, _) = read(&bytes, OutputLevel::Java8, limits());
    assert!(facts.concat.is_empty());
    assert_eq!(codes(&facts), vec!["classfile_concat_handle_unresolved"]);
    assert!(
        facts.diagnostics[0]
            .message
            .contains("no factory can be named")
    );
    assert!(facts.condy.is_complete());
    assert_eq!(facts.condy.use_sites.len(), 1);
}

// ---------------------------------------------------------------------------
// The output-level answer
// ---------------------------------------------------------------------------

#[test]
fn java8_conflicts_with_record_sealed_and_concat_and_keeps_their_origins() {
    // Record components: the attribute rule decides the release the construct needs.
    let (record, _) = read(RECORD_FIXTURE, OutputLevel::Java8, limits());
    let status = record.output_level.clone();
    let OutputLevelStatus::Conflict { level, conflicts } = status else {
        panic!("a record cannot be represented at the Java 8 output level: {status:?}");
    };
    assert_eq!(level, OutputLevel::Java8);
    assert_eq!(conflicts.len(), 1, "{conflicts:#?}");
    assert_eq!(conflicts[0].feature, ModernFeature::RecordComponents);
    assert_eq!(conflicts[0].since, 60);
    assert_eq!(conflicts[0].source, "JVMS 4.7.30");
    match &conflicts[0].origin {
        ModernOrigin::ClassAttribute {
            name,
            name_index,
            span,
            content_span,
        } => {
            assert_eq!(name.0, b"Record".to_vec());
            assert_eq!(
                *name_index,
                record.record.as_ref().unwrap().name_index,
                "the conflict's origin is the entry's own attribute_name_index"
            );
            assert_eq!(span, &record.record.as_ref().unwrap().span);
            assert_eq!(content_span, &record.record.as_ref().unwrap().content_span);
        }
        other => panic!("a record's origin is its own attribute entry: {other:?}"),
    }
    // The fallback: the fact is still published, with its components and their origins, instead of
    // being rewritten into something the Java 8 output level could print.
    assert_eq!(
        record.record.as_ref().unwrap().components.len(),
        3,
        "the conflicting fact keeps every component"
    );
    assert_eq!(
        record.assess_output_level(OutputLevel::Java8),
        record.output_level
    );

    // Permitted subclasses: the sealed declaration.
    let (sealed, _) = read(SEALED_FIXTURE, OutputLevel::Java8, limits());
    let OutputLevelStatus::Conflict { conflicts, .. } = sealed.output_level.clone() else {
        panic!("a sealed class cannot be represented at the Java 8 output level");
    };
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].feature, ModernFeature::PermittedSubclasses);
    assert_eq!(conflicts[0].since, 61);
    assert!(sealed.permitted_subclasses.is_some());

    // Modern concat: one conflict per site, each naming the site's own constant-pool entry.
    let (concat, _) = read(CONCAT_FIXTURE, OutputLevel::Java8, limits());
    let OutputLevelStatus::Conflict { conflicts, .. } = concat.output_level.clone() else {
        panic!("a StringConcatFactory site cannot be represented at the Java 8 output level");
    };
    assert_eq!(conflicts.len(), 2, "{conflicts:#?}");
    for (conflict, site) in conflicts.iter().zip(&concat.concat) {
        assert_eq!(conflict.feature, ModernFeature::StringConcat);
        assert_eq!(conflict.since, 51);
        assert_eq!(
            conflict.origin,
            ModernOrigin::ConstantPoolEntry {
                index: site.constant_pool_index,
                span: site.span.clone(),
            }
        );
    }
    assert_eq!(
        concat.concat.len(),
        2,
        "the fallback keeps both sites as facts"
    );
}

#[test]
fn a_java8_class_has_no_output_level_conflict() {
    let (facts, _) = read(JAVA8_FIXTURE, OutputLevel::Java8, limits());
    assert!(facts.record.is_none());
    assert!(facts.permitted_subclasses.is_none());
    assert!(facts.concat.is_empty());
    assert_eq!(
        facts.output_level,
        OutputLevelStatus::Representable {
            level: OutputLevel::Java8
        },
        "nothing modern is declared, so the Java 8 output level represents all of it"
    );
    assert_eq!(
        facts.assess_output_level(OutputLevel::Java8),
        facts.output_level
    );
}

#[test]
fn a_construct_no_pass_judges_gets_no_output_level_verdict() {
    // The vocabulary is closed over the constructs a pass really evaluates: a new variant compiles
    // only once it is written here, i.e. once someone has named the fact that produces it. Design
    // decision 5 names exactly these three.
    for feature in [
        ModernFeature::RecordComponents,
        ModernFeature::PermittedSubclasses,
        ModernFeature::StringConcat,
    ] {
        match feature {
            ModernFeature::RecordComponents
            | ModernFeature::PermittedSubclasses
            | ModernFeature::StringConcat => {}
        }
    }

    // A nestmate declaration and a module descriptor are modern structures this pass places (the P1
    // fact reader owns their content), and neither of them is one of the judged constructs: the Java
    // 8 answer states no conflict for them instead of inventing a verdict no pass produced.
    let (nest, _) = read(NEST_FIXTURE, OutputLevel::Java8, limits());
    let members = nest
        .attributes
        .iter()
        .find(|row| row.name.0 == b"NestMembers")
        .expect("NestMembers is placed");
    assert!(!members.read, "the P1 fact reader owns the nestmate fact");
    assert_eq!(
        nest.output_level,
        OutputLevelStatus::Representable {
            level: OutputLevel::Java8
        },
        "a nestmate declaration is not one of the constructs this pass judges"
    );

    let (module, _) = read(MODULE_FIXTURE, OutputLevel::Java8, limits());
    let descriptor = module
        .attributes
        .iter()
        .find(|row| row.name.0 == b"Module")
        .expect("Module is placed");
    assert!(!descriptor.read, "the P1 fact reader owns the module fact");
    assert_eq!(
        module.output_level,
        OutputLevelStatus::Representable {
            level: OutputLevel::Java8
        },
        "a module descriptor is not one of the constructs this pass judges"
    );

    // The constant-dynamic graph: a complete graph over a handle that names a class no file here
    // declares, read under the Java 8 level, and still no conflict — the graph is a deferred fact,
    // not a creation this level would have to represent.
    let (bytes, _) = condy_fixture();
    let (condy, _) = read(&bytes, OutputLevel::Java8, limits());
    assert!(condy.condy.is_complete());
    assert_eq!(
        condy.output_level,
        OutputLevelStatus::Representable {
            level: OutputLevel::Java8
        },
        "a constant-dynamic graph is not one of the constructs this pass judges"
    );
}

#[test]
fn the_header_plane_still_evaluates_no_output_level_and_applies_no_attribute_rule() {
    let inspection = inspect_header(
        CONCAT_FIXTURE,
        &mut Budget::new(limits()),
        InspectionMode::Forensic,
    )
    .unwrap();
    assert_eq!(
        inspection.output_level,
        OutputLevelStatus::NotEvaluated,
        "the header plane states version facts, and the pass that reads the facts states this one"
    );
    assert!(
        !inspection
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.starts_with("classfile_attribute_")),
        "no attribute legality belongs to header inspection: {:?}",
        inspection
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect::<Vec<_>>()
    );
}
