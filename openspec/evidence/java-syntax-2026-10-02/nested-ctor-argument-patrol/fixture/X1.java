public class X1 {
    public static String wrapCtor(String s) {
        try {
            if (s.isEmpty()) throw new IllegalArgumentException("empty");
            return s;
        } catch (IllegalArgumentException e) {
            RuntimeException r = new RuntimeException("wrapped", e);
            throw r;
        }
    }
    public static String wrapInitCause(String s) {
        try {
            if (s.isEmpty()) throw new IllegalStateException("bad");
            return s;
        } catch (IllegalStateException e) {
            e.initCause(new UnsupportedOperationException("root"));
            throw e;
        }
    }
    public static String readCause(Throwable t) {
        Throwable c = t.getCause();
        return c == null ? "none" : c.getMessage();
    }
    public static void main(String[] a) {
        try { wrapCtor(""); } catch (RuntimeException e) { System.out.println(readCause(e)); }
        try { wrapInitCause(""); } catch (IllegalStateException e) { System.out.println(readCause(e)); }
        System.out.println(readCause(new Exception("top", new Exception("inner"))));
    }
}
