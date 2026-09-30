public class C1x {
    public static void ns(int mode) { if (mode != 0) throw new NoSuchFieldError("injected"); }
    public static void is(int mode) { if (mode != 0) throw new IllegalStateException("injected"); }
    public static void re(int mode) { if (mode != 0) throw new RuntimeException("injected"); }
    public static int work(int x) { return x + 1; }
    public static String swallow(int mode) {
        try {
            ns(mode);
            work(1);
            return "ok";
        } catch (NoSuchFieldError unused) {
        }
        return "missed";
    }
    public static String five(int mode) {
        StringBuilder b = new StringBuilder();
        try { ns(mode); b.append('a'); } catch (NoSuchFieldError unused) { }
        try { b.append('b'); } catch (NoSuchFieldError unused) { }
        try { is(mode); b.append('c'); } catch (IllegalStateException unused) { }
        try { b.append('d'); } catch (NoSuchFieldError unused) { }
        try { re(mode); b.append('e'); } catch (RuntimeException unused) { }
        return b.toString();
    }
    public static void main(String[] args) {
        System.out.println(swallow(args.length));
        System.out.println(five(args.length));
    }
}
