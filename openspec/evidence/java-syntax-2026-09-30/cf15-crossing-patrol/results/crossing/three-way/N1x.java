public class N1x {
    public static java.io.ByteArrayOutputStream open() { return new java.io.ByteArrayOutputStream(); }
    public static void t(int mode) { if (mode != 0) throw new IllegalStateException("state"); }
    public static void main(String[] args) {
        run(args.length);
    }
    public static void run(int mode) {
        java.io.ByteArrayOutputStream r = open();
        try {
            t(mode);
            r.write(1);
        } catch (IllegalStateException error) {
            System.out.println(r.size());
        }
        System.out.println("end:" + r.size());
    }
}
