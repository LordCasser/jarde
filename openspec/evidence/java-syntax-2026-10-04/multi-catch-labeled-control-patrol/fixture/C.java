public class C {
    // continue outer, 但内层循环**不是**外层体的最后一条语句（后面还有 println）
    static String trail() {
        StringBuilder sb = new StringBuilder();
        outer: for (int i = 0; i < 2; i++) {
            for (int j = 0; j < 2; j++) {
                if (j == 1) continue outer;
                sb.append("in" + i + j + ";");
            }
            sb.append("after" + i + ";");   // ← 若 continue outer 被错译为 break，此行会被执行
        }
        return sb.toString();
    }
    // 对照：break outer（内层后仍有语句）—— 语义本应跳过 after
    static String trailBreak() {
        StringBuilder sb = new StringBuilder();
        outer: for (int i = 0; i < 2; i++) {
            for (int j = 0; j < 2; j++) {
                if (j == 1) break outer;
                sb.append("in" + i + j + ";");
            }
            sb.append("after" + i + ";");
        }
        return sb.toString();
    }
    public static void main(String[] a) {
        System.out.println("trail=" + trail());
        System.out.println("trailBreak=" + trailBreak());
    }
}
