public class M5 {
    public static void helper() { System.out.println("helper"); }
    public static void t() { throw new IllegalStateException("state"); }
    public static void u(IllegalStateException e) { System.out.println(e.getMessage()); }
    public static void main(String[] args) {
        helper();
        try {
            t();
            System.out.println("missing throw");
        } catch (IllegalStateException error) {
            u(error);
        }
    }
}
