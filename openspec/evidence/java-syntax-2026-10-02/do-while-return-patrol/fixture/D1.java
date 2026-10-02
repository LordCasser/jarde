public class D1 {
    static int hits;
    public static String retInDoWhile() {
        int i = 0;
        while (i < 10) {
            i++;
            do {
                if (i % 3 == 0) break;
                if (i > 7) return "early:" + hits;
                hits++;
            } while (false);
        }
        return "i=" + i;
    }
    public static String retInPlainDo() {
        int i = 0;
        do {
            if (i == 3) return "d3";
            i++;
        } while (i < 5);
        return "d" + i;
    }
    public static String retInIf() {
        for (int i = 0; i < 5; i++) {
            if (i == 2) return "f2";
        }
        return "end";
    }
    public static void main(String[] a) {
        System.out.println(retInDoWhile());
        System.out.println(retInPlainDo());
        System.out.println(retInIf());
    }
}
