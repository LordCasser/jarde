// The runtime-only superclass: present on the javac/java classpath, excluded from the frozen
// snapshot jar.  `H2$Ext` reaches `H2$Target` only through this class, so a recovery whose
// snapshot holds `H2$Ext` and `H2$Target` but not `Helper` has no in-snapshot chain and must
// keep the refusal, while the original still verifies and runs with `Helper` on the classpath.
public class Helper implements H2.Target {
    public String tag() { return "helper"; }
}
