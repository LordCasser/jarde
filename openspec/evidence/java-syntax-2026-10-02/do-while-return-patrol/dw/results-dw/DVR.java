public class DVR {
    public static String doubleReturnDoWhile() {
        int local0;
        local0 = 0;
        while (local0 < 10) {
            local0 = local0 + 1;
            if (local0 % 3 == 0) {
            } else if (local0 > 7) {
                return "a" + local0;
            } else if (local0 > 8) {
                return "b" + local0;
            }
        }
        return "i=" + local0;
    }
    public static void main(String[] a) { System.out.println(doubleReturnDoWhile()); }
}
