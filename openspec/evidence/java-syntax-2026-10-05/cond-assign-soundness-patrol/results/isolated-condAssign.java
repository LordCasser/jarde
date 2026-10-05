public class CA {
static boolean condAssign(boolean[] xs) {
        int local1;
        int local3;
        local1 = 0;
        local3 = 0;
        while (local3 < xs.length) {
            local3 = local3 + 1;
        }
        return local1 % 2 != 0;
    }

    public static void main(String[] a){ System.out.println(CA.condAssign(new boolean[]{false,true})); }
}
