public class Nest {
    static int nest(Runnable r) {
        try {
            try {
                r.run();
            } catch (IllegalArgumentException e) {
                return -1;
            }
        } catch (RuntimeException e) {
            return -2;
        }
        return 0;
    }
}
