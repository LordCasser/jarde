public class BI2 {
    boolean ok = false;
    boolean earlyRet(int[] arg1) {
        int[] local2;
        local2 = arg1;
        for (int local5 : local2) {
            if (!this.ok) {
                return false;
            }
        }
        return true;
    }

    public static void main(String[] a){ BI2 b = new BI2(); b.ok = true; System.out.println(b.earlyRet(new int[]{1,-2,3})); }
}
