public final class LoopAndRecovered {
    public static int andWhile(int arg0, int arg1) {
        int local2;
        local2 = 0;
        while (arg0 > 0) {
            if (arg1 > 0) {
                local2 = local2 + arg0;
                arg0 = arg0 - 1;
                arg1 = arg1 - 1;
            }
        }
        return local2;
    }

    public static void main(String[] args) {
        System.out.println(andWhile(1, 0));
    }
}
