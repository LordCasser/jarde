public final class PostfixHandlerBoundary {
    static int[] array = new int[] { 5 };
    static boolean failIndex;
    static int calls;

    static int[] a() {
        calls++;
        return array;
    }

    static int i() {
        calls++;
        if (failIndex) {
            throw new IllegalArgumentException("index");
        }
        return 0;
    }

    static int update() {
        try {
            return a()[i()]++;
        } catch (RuntimeException ex) {
            return -1;
        }
    }

    public static void main(String[] args) {
        failIndex = true;
        try {
            update();
            System.out.println("escaped=none");
        } catch (RuntimeException ex) {
            System.out.println("escaped=" + ex.getClass().getSimpleName());
        }
        System.out.println("calls=" + calls);
    }
}
