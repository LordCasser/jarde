public class PD implements AutoCloseable {
    static StringBuilder log = new StringBuilder();
    static void touch(PD r) { log.append("t"); }
    @Override public void close() { log.append("[c]"); }

    // soloFin: the patrol's P3.voidBodySoloFin form in its own class (`t[c]f`), the shape the
    // direct certificate is anchored on — and the base of the two patched negatives.
    public static String soloFin() throws Exception {
        try (PD r = new PD()) {
            touch(r);
        } finally {
            log.append("f");
        }
        return log.toString();
    }

    // finReturn: the finally clause itself returns (degenerate form — the exceptional copy ends in
    // the return, not a rethrow; registered, not claimed).
    public static String finReturn() throws Exception {
        try (PD r = new PD()) {
            touch(r);
        } finally {
            return log.toString();
        }
    }

    // multiFin: two TWR levels with the outer statement carrying the finally (one form verified,
    // current behaviour registered, not generalized).
    public static String multiFin() throws Exception {
        try (PD a = new PD()) {
            try (PD b = new PD()) {
                touch(b);
            }
        } finally {
            log.append("f");
        }
        return log.toString();
    }

    // bodyReturn: the guarded body returns through the clause — the saved value's load/return pair
    // follows the copy the finally placed.
    public static String bodyReturn() throws Exception {
        try (PD r = new PD()) {
            touch(r);
            return log.toString();
        } finally {
            log.append("f");
        }
    }

    // threeClauses: TWR + catch + finally — three clauses, out of this change's scope; the whole
    // method keeps its refusal.
    public static String threeClauses() throws Exception {
        try (PD r = new PD()) {
            touch(r);
        } catch (IllegalStateException e) {
            log.append("E");
        } finally {
            log.append("f");
        }
        return log.toString();
    }

    public static void main(String[] a) throws Exception {
        System.out.println(soloFin());
        System.out.println(finReturn());
        System.out.println(multiFin());
        System.out.println(bodyReturn());
        System.out.println(threeClauses());
    }
}
