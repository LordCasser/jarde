public class TF implements AutoCloseable {
    static StringBuilder log = new StringBuilder();
    String name;
    TF(String n) { name = n; }
    @Override public void close() { log.append('[').append(name).append(']'); }

    // v1: outer TWR + inner explicit finally, no inner TWR
    public static String v1() throws Exception {
        try (TF a = new TF("a")) {
            try { log.append("body"); } finally { log.append("mid"); }
        }
        return log.toString();
    }

    // v2: double TWR, no finally (regression anchor)
    public static String v2() throws Exception {
        try (TF a = new TF("a")) {
            try (TF b = new TF("b")) { log.append("body"); }
        }
        return log.toString();
    }

    // v3: outer TWR + inner finally whose guarded body returns
    public static int v3() throws Exception {
        try (TF a = new TF("a")) {
            try { log.append("body"); return log.length(); } finally { log.append("mid"); }
        }
    }

    // nested: the patrol's T4.nested shape itself
    public static String nested() throws Exception {
        try (TF a = new TF("a")) {
            try (TF b = new TF("b")) { log.append("body"); } finally { log.append("mid"); }
        }
        return log.toString();
    }

    // solo: TWR with its own finally, no enclosing TWR (scope probe: not this slice's claim)
    public static String solo() throws Exception {
        try (TF b = new TF("b")) { log.append("body"); } finally { log.append("mid"); }
        return log.toString();
    }

    public static void main(String[] x) throws Exception {
        log.setLength(0);
        System.out.println(v1());
        log.setLength(0);
        System.out.println(v2());
        log.setLength(0);
        System.out.println(v3());
        log.setLength(0);
        System.out.println(nested());
        log.setLength(0);
        System.out.println(solo());
    }
}
