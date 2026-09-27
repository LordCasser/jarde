package cf08join;

public final class LoopIfJoin {
    public static int run(boolean outer, boolean inner) {
        int value = 0;
        if (outer) {
            if (inner) {
                value = 1;
            } else {
                while (value < 3) {
                    value++;
                }
            }
            value += 10;
        } else {
            value = 4;
        }
        return value;
    }

    public static void main(String[] args) {
        System.out.println(run(false, false));
        System.out.println(run(true, false));
        System.out.println(run(true, true));
    }
}
