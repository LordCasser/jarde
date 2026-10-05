public class NL extends java.lang.Object {
    static final java.lang.Object LOCK = new java.lang.Object();

    public NL() {
        super();
        return;
    }

    static java.lang.String probe(java.lang.Object arg0) {
        synchronized (NL.LOCK) {
            synchronized (NL.class) {
            }
            return "n" + arg0;
        }
    }

    public static void main(java.lang.String[] arg0) {
        java.lang.System.out.println((java.lang.String) probe((java.lang.Object) new Box()));
        return;
    }

    static class Box extends java.lang.Object {
        Box() {
            super();
            return;
        }

        public java.lang.String toString() {
            return java.lang.Thread.holdsLock((java.lang.Object) NL.class) ? "Y" : "N";
        }
    }
}
