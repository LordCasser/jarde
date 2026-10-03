public class WCallI1 {
    interface M { static int sv() { return 8; } int v(); }
    public static void main(String[] a) { System.out.println(M.sv()); }
}
