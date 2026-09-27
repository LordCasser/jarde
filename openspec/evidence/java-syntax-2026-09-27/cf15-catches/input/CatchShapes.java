public class CatchShapes {
    public static String ordinary(int kind) {
        try {
            if (kind == 1) throw new IllegalArgumentException("arg");
            if (kind == 2) throw new IllegalStateException("state");
            return "ok";
        } catch (IllegalArgumentException error) {
            return "argument:" + error.getMessage();
        } catch (IllegalStateException error) {
            return "state:" + error.getMessage();
        }
    }

    public static String multi(boolean state) {
        try {
            if (state) throw new IllegalStateException("multi");
            throw new IllegalArgumentException("multi");
        } catch (IllegalArgumentException | IllegalStateException error) {
            return error.getClass().getSimpleName() + ":" + error.getMessage();
        }
    }

    public static void main(String[] args) {
        System.out.println(ordinary(0));
        System.out.println(ordinary(1));
        System.out.println(ordinary(2));
        System.out.println(multi(false));
        System.out.println(multi(true));
    }
}
