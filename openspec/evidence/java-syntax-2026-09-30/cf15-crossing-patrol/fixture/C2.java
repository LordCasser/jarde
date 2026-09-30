public class C2 {
    public static String alias(int mode) {
        try {
            if (mode == 0) throw new IllegalStateException("zero");
            return "fine";
        } catch (IllegalStateException e) {
            String m = e.getMessage();
            RuntimeException wrapped = new RuntimeException("w:" + m, e);
            throw wrapped;
        }
    }
    public static void main(String[] args) {
        try { alias(0); } catch (RuntimeException e) { System.out.println(e.getMessage() + "/" + (e.getCause() instanceof IllegalStateException)); }
        System.out.println(alias(1));
    }
}
