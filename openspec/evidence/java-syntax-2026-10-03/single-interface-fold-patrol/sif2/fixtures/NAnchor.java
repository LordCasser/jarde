public class NAnchor {
    interface M { static int sv() { return 8; } }
    static M pick(M candidate) { M local = candidate; return local; }
    public static void main(String[] a) {
        System.out.println(M.sv() + (pick(null) == null ? 1 : 0));
    }
}
