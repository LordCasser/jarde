public class TP {
static int[] tp(int[] arg0) {
        int local1;
        int local2;
        local1 = 0;
        local2 = 0;
        while (local2 < arg0.length) {
            if (arg0[local2] != 0) {
                local1 = local1 + 1;
            }
            local2 = local2 + 1;
        }
        return java.util.Arrays.copyOf(arg0, local1);
    }

    public static void main(String[] a){ System.out.println(java.util.Arrays.toString(tp(new int[]{0,1,0,2,3,0}))); }
}
