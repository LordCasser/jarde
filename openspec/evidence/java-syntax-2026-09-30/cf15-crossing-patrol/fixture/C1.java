public class C1 {
    public static int work(int x) { return x + 1; }
    public static String swallow() {
        try {
            work(1);
            return "ok";
        } catch (NoSuchFieldError unused) {
        }
        return "missed";
    }
    public static String five() {
        StringBuilder b = new StringBuilder();
        try { b.append('a'); } catch (NoSuchFieldError unused) { }
        try { b.append('b'); } catch (NoSuchFieldError unused) { }
        try { b.append('c'); } catch (IllegalStateException unused) { }
        try { b.append('d'); } catch (NoSuchFieldError unused) { }
        try { b.append('e'); } catch (RuntimeException unused) { }
        return b.toString();
    }
    public static void main(String[] args) {
        System.out.println(swallow());
        System.out.println(five());
    }
}
