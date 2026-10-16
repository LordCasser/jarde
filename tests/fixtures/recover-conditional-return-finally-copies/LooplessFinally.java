public class LooplessFinally {
    static int value;
    static int loopless(int[] xs) {
        try {
            if (xs.length > 0 && xs[0] == 3) return value;
        } finally {
            System.out.println("f");
        }
        return -1;
    }
}
