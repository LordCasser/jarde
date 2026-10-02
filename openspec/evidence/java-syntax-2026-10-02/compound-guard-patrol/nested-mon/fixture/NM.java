public class NM {
    static StringBuilder log = new StringBuilder();
    // V1: T4 form — different lock objects, loop inside inner, return of outer
    public static int v1(int n) {
        synchronized (NM.class) {
            int s = 0;
            synchronized (log) {
                for (int i = 0; i < n; i++) { s += i; }
            }
            return s;
        }
    }
    // V2: inner synchronized inside the outer's loop
    public static int v2(int n) {
        synchronized (NM.class) {
            int s = 0;
            for (int i = 0; i < n; i++) {
                synchronized (log) { s += i; }
            }
            return s;
        }
    }
    // V3: the inner synchronized's body ends in the return
    public static int v3(int n) {
        synchronized (NM.class) {
            int s = 0;
            synchronized (log) {
                for (int i = 0; i < n; i++) { s += i; }
                return s;
            }
        }
    }
    // N3: three levels — stays refused
    public static int n3(int n) {
        synchronized (NM.class) {
            int s = 0;
            synchronized (log) {
                synchronized (new Object()) {
                    s += n;
                }
            }
            return s;
        }
    }
    // SR: same-lock reentrant — register current behavior
    public static int sr(Object a, int n) {
        synchronized (a) {
            int s = 0;
            synchronized (a) {
                for (int i = 0; i < n; i++) { s += i; }
            }
            return s;
        }
    }
    public static void main(String[] x) {
        System.out.println(v1(5));
        System.out.println(v2(4));
        System.out.println(v3(5));
        System.out.println(n3(5));
        System.out.println(sr(new Object(), 5));
    }
}
