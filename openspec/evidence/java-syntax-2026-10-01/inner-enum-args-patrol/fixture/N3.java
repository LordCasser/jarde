public class N3 {
    public enum Simple {
        A((byte) 1, "x"), B((byte) 2, "y");
        private final byte num;
        private final String s;
        Simple(byte n, String s) { this.num = n; this.s = s; }
        public int n() { return num; }
    }
    public enum Refs {
        A(Simple.A), B(Simple.B);
        private final Simple s;
        Refs(Simple s) { this.s = s; }
        public Simple g() { return s; }
    }
    public static void main(String[] a) {
        System.out.println(N3.Simple.A.n());
        System.out.println(N3.Refs.A.g().n());
    }
}
