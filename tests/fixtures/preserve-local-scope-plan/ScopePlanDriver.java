/// The three-way driver (`preserve-local-scope-across-exception-regions` 2.3): one call per member
/// of `ScopePlan` on its normal and exceptional inputs, then one call whose `Error` escapes both
/// clauses, so the compared trace holds return values *and* an escaping exception's own type and
/// message.
///
/// The same source is compiled beside the fixture's own class, beside the frozen JADX render (with
/// `package defpackage;` prepended, the package that render states) and beside the recovered text,
/// and each side is run under `java -Xverify:all`. The three sides must print the same trace.
public class ScopePlanDriver {
    public static void main(String[] args) {
        System.out.println("catchOnly false=" + ScopePlan.catchOnly(false));
        System.out.println("catchOnly true=" + ScopePlan.catchOnly(true));
        System.out.println("assignedAcrossTry false=" + ScopePlan.assignedAcrossTry(false));
        System.out.println("assignedAcrossTry true=" + ScopePlan.assignedAcrossTry(true));
        System.out.println("assignedAcrossIf false=" + ScopePlan.assignedAcrossIf(false));
        System.out.println("assignedAcrossIf true=" + ScopePlan.assignedAcrossIf(true));
        System.out.println("nestedHandlerOnly null=" + ScopePlan.nestedHandlerOnly(null));
        System.out.println("nestedAcross false=" + ScopePlan.nestedAcross(false));
        System.out.println("nestedAcross true=" + ScopePlan.nestedAcross(true));
        Runnable boom = new Runnable() {
            public void run() {
                throw new AssertionError("boom");
            }
        };
        try {
            ScopePlan.nestedHandlerOnly(boom);
            System.out.println("nestedHandlerOnly error=none");
        } catch (Throwable escaped) {
            System.out.println(
                    "nestedHandlerOnly error="
                            + escaped.getClass().getName()
                            + ":"
                            + escaped.getMessage());
        }
    }
}
