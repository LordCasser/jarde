import java.util.*;

/// Mechanical check of the `platform_reference_argument_widens` java.util table in
/// `crates/jarde-java/src/build.rs` against the running JDK's own reflection. Run under the real
/// JDK 8 (Corretto 1.8.0_432) so the `extends`/`implements` facts are the release-8 ones — the
/// table is release-gated for a reason: on JDK 21+ `LinkedHashSet implements SequencedSet` and
/// `TreeSet`/`LinkedHashMap` follow, so the release-8 rows are no longer direct edges there. A run
/// under a later JDK is expected to fail check A on the first such row; the record of that
/// divergence is the evidence for the `java_release == 8` gate.
///
/// The rows are read off stdin, one `child parent` pair per line. The checks are:
///
///   A. every row is a *direct edge* the runtime states — a class child's parent is its
///      `getSuperclass()` or one of its `getInterfaces()`, an interface child's parent is one of
///      its `getInterfaces()`;
///   B. every row's child is a `java.util` collection type and its parent is `java.util` (the only
///      exception being the `Collection -> java.lang.Iterable` root edge);
///   C. the table's reachability, over the enumerated domain, equals the runtime's own
///      assignability over that domain — a missing row makes the two disagree;
///   D. the rows are exactly the domain-internal direct edges of the domain types — no extra row,
///      no missing one (a parent outside the enumerated domain, such as `Hashtable`'s
///      `java.util.Dictionary`, is intentionally not a row);
///   E. types outside the enumerated domain (the `java.util` subpackages and the collection /
///      non-collection `java.util` types this slice leaves out) stay unreachable.
public class CollectionTableCheck {
    /// The table rows, `child parent`.
    static final List<String[]> ROWS = new ArrayList<>();
    /// Every type the table names.
    static final Set<String> DOMAIN = new LinkedHashSet<>();
    /// `java.util` collection implementations the slice leaves out of the table, plus one
    /// non-collection `java.util` type and the `Dictionary` parent of `Hashtable`.
    static final String[] TABLE_OUT = {
        "java.util.EnumSet",
        "java.util.IdentityHashMap",
        "java.util.PriorityQueue",
        "java.util.WeakHashMap",
        "java.util.EnumMap",
        "java.util.AbstractQueue",
        "java.util.Dictionary",
        "java.util.Date",
        "java.util.Collections",
        "java.util.Arrays",
        "java.util.concurrent.ConcurrentHashMap",
        "java.util.concurrent.CopyOnWriteArrayList",
    };

    static Class<?> load(String name) {
        try {
            return Class.forName(name, false, ClassLoader.getSystemClassLoader());
        } catch (ClassNotFoundException e) {
            throw new AssertionError("the runtime does not define " + name, e);
        }
    }

    /// The runtime's direct parents of a type: the superclass plus the interfaces for a class, the
    /// interfaces for an interface.
    static Set<String> directParents(String name) {
        Class<?> c = load(name);
        Set<String> parents = new LinkedHashSet<>();
        if (!c.isInterface() && c.getSuperclass() != null) {
            parents.add(c.getSuperclass().getName());
        }
        for (Class<?> i : c.getInterfaces()) {
            parents.add(i.getName());
        }
        return parents;
    }

    /// Mirrors the Rust walk: layered breadth-first along the rows, capped at the row count, with
    /// each parent compared as it is appended. A type equal to the presented one is not a widening.
    static boolean tableReaches(String presented, String required) {
        List<String> frontier = new ArrayList<>();
        frontier.add(presented);
        for (int step = 0; step < ROWS.size(); step++) {
            List<String> next = new ArrayList<>();
            for (String current : frontier) {
                for (String[] row : ROWS) {
                    if (row[0].equals(current)) {
                        if (row[1].equals(required)) {
                            return true;
                        }
                        next.add(row[1]);
                    }
                }
            }
            if (next.isEmpty()) {
                return false;
            }
            frontier = next;
        }
        return false;
    }

    /// The runtime's own assignability restricted to the enumerated domain (excluding the
    /// same-type pair, which the dispatch answers before the table).
    static boolean runtimeReaches(String presented, String required, Set<String> domain) {
        if (presented.equals(required)) {
            return false;
        }
        Class<?> from = load(presented);
        Class<?> to = load(required);
        return to.isAssignableFrom(from);
    }

    public static void main(String[] args) {
        Scanner in = new Scanner(System.in);
        while (in.hasNextLine()) {
            String line = in.nextLine().trim();
            if (line.isEmpty()) {
                continue;
            }
            String[] parts = line.split("\\s+");
            if (parts.length != 2) {
                throw new AssertionError("malformed row: " + line);
            }
            ROWS.add(parts);
            DOMAIN.add(parts[0]);
            DOMAIN.add(parts[1]);
        }
        if (ROWS.isEmpty()) {
            throw new AssertionError("no rows were read");
        }
        System.out.println("read " + ROWS.size() + " rows over " + DOMAIN.size() + " types");

        // A: each row is a real direct edge of the running JDK.
        for (String[] row : ROWS) {
            Set<String> parents = directParents(row[0]);
            if (!parents.contains(row[1])) {
                throw new AssertionError(
                    "not a direct edge of this JDK: " + row[0] + " -> " + row[1] + " (runtime: " + parents + ")");
            }
        }
        System.out.println("A ok: every row is a direct superclass/interface edge of the runtime");

        // B: the package boundary.
        for (String[] row : ROWS) {
            if (!row[0].startsWith("java.util.")) {
                throw new AssertionError("row child outside java.util: " + row[0]);
            }
            boolean isIterableRoot = row[1].equals("java.lang.Iterable");
            if (!row[1].startsWith("java.util.") && !isIterableRoot) {
                throw new AssertionError("row parent outside the closed set: " + row[1]);
            }
            if (isIterableRoot && !row[0].equals("java.util.Collection")) {
                throw new AssertionError("only Collection may name the Iterable root: " + row[0]);
            }
        }
        System.out.println("B ok: every row stays in java.util (with the one Collection -> Iterable root)");

        // C: reachability over the domain equals runtime assignability over the domain.
        int pairs = 0;
        for (String presented : DOMAIN) {
            for (String required : DOMAIN) {
                if (presented.equals(required)) {
                    continue;
                }
                pairs++;
                boolean table = tableReaches(presented, required);
                boolean runtime = runtimeReaches(presented, required, DOMAIN);
                if (table != runtime) {
                    throw new AssertionError(
                        "table/runtime disagree: " + presented + " -> " + required + " (table=" + table + ", runtime=" + runtime + ")");
                }
            }
        }
        System.out.println("C ok: over " + pairs + " domain pairs, the walk equals the runtime's own assignability");

        // D: the rows are exactly the domain-internal direct edges.
        Set<String> stated = new LinkedHashSet<>();
        for (String[] row : ROWS) {
            stated.add(row[0] + " " + row[1]);
        }
        Set<String> expected = new LinkedHashSet<>();
        for (String type : DOMAIN) {
            for (String parent : directParents(type)) {
                if (DOMAIN.contains(parent) || parent.equals("java.lang.Iterable")) {
                    expected.add(type + " " + parent);
                }
            }
        }
        if (!stated.equals(expected)) {
            Set<String> missing = new LinkedHashSet<>(expected);
            missing.removeAll(stated);
            Set<String> extra = new LinkedHashSet<>(stated);
            extra.removeAll(expected);
            throw new AssertionError("rows differ from the domain's direct edges: missing=" + missing + " extra=" + extra);
        }
        System.out.println("D ok: the " + stated.size() + " rows are exactly the domain-internal direct edges, none missing, none extra");

        // E: the types this slice leaves out never enter the table. `Dictionary` is deliberately
        // the sharp case: it *is* a runtime superclass of `Hashtable`, and the table intentionally
        // states no such edge, so the walk must not reach it from any domain type. Every other
        // table-out type is either not a domain supertype at all or an interface a domain type
        // would need a row for.
        int boundaryChecks = 0;
        for (String outside : TABLE_OUT) {
            load(outside);
            if (DOMAIN.contains(outside)) {
                throw new AssertionError("table-out type is a table node: " + outside);
            }
            for (String presented : DOMAIN) {
                if (tableReaches(presented, outside)) {
                    throw new AssertionError("the walk reached table-out type " + outside + " from " + presented);
                }
                boundaryChecks++;
            }
        }
        // The one runtime hierarchy the table declines on purpose: a `Hashtable`/`Properties` value
        // is assignable to its `Dictionary` parent, and the table still refuses it.
        if (!runtimeReaches("java.util.Hashtable", "java.util.Dictionary", DOMAIN)) {
            throw new AssertionError("Dictionary is not Hashtable's runtime superclass; the sharp case is gone");
        }
        if (tableReaches("java.util.Hashtable", "java.util.Dictionary")) {
            throw new AssertionError("the table must not state Hashtable -> Dictionary");
        }
        System.out.println("E ok: " + TABLE_OUT.length + " table-out types never enter the walk (" + boundaryChecks + " from-domain pairs), and Hashtable -> Dictionary stays refused though the runtime allows it");
        System.out.println("ALL CHECKS PASSED");
    }
}