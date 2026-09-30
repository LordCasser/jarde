public class S1V implements AutoCloseable {
    @Override public void close() { }
    static String tagA(IllegalStateException e) { return "A:" + e.getMessage(); }
    static String tagB(IllegalArgumentException e) { return "B:" + e.getMessage(); }
    static String tagR(RuntimeException e) { return "R:" + e.getMessage(); }
    static void touch(S1V r) { }
    // V1: two separate named catch clauses, each parameter reusing the resource slot.
    public static String clausesHelper() throws Exception {
        try (S1V r = new S1V()) { touch(r); }
        catch (IllegalStateException ex1) { return tagA(ex1); }
        catch (IllegalArgumentException ex2) { return tagB(ex2); }
        return "done";
    }
    // V2: one multi-catch clause reusing the resource slot.
    public static String multiHelper() throws Exception {
        try (S1V r = new S1V()) { touch(r); }
        catch (IllegalStateException | IllegalArgumentException both) { return tagR(both); }
        return "done";
    }
    // V3: handler stores a copy first, then passes the copy.
    public static String copyHelper() throws Exception {
        try (S1V r = new S1V()) { touch(r); }
        catch (IllegalStateException e) { IllegalStateException copy = e; return tagA(copy); }
        return "done";
    }
    // V4: non-handler slot reuse (scope-driven), no catch involved.
    public static String scopeHelper() throws Exception {
        {
            S1V r = new S1V();
            touch(r);
        }
        IllegalStateException later = new IllegalStateException("x");
        return tagA(later);
    }
    // V5: rethrow from a slot-reusing handler (existing rethrow channel must not change).
    public static void rethrowHelper() throws Exception {
        try (S1V r = new S1V()) { touch(r); }
        catch (IllegalStateException e) { throw e; }
    }
    // Control: two named clauses without slot reuse (no TWR).
    public static String plainClauses() throws Exception {
        try { touch(null); }
        catch (IllegalStateException ex1) { return tagA(ex1); }
        catch (IllegalArgumentException ex2) { return tagB(ex2); }
        return "done";
    }
    public static void main(String[] a) throws Exception {
        System.out.println(clausesHelper());
        System.out.println(multiHelper());
        System.out.println(copyHelper());
        System.out.println(scopeHelper());
        try { rethrowHelper(); } catch (IllegalStateException caught) { System.out.println("rethrown:" + caught.getMessage()); }
        System.out.println(plainClauses());
    }
}
