public class C4x implements AutoCloseable {
    @Override public void close() { }
    public static void is(int mode) { if (mode != 0) throw new IllegalStateException("injected"); }
    public static String constructNamed(int mode) {
        StringBuilder b = new StringBuilder();
        try {
            is(mode);
            b.append('a');
        } catch (IllegalStateException e) {
            return "caught";
        }
        return b.toString();
    }
    public static void main(String[] args) {
        System.out.println(constructNamed(args.length));
    }
}
