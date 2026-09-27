package dt13;

public final class NestedShape {
    public enum Major {
        FIRST, SECOND;

        public enum Minor {
            LEFT, RIGHT
        }
    }

    public static String observe() {
        return Major.FIRST + ":" + Major.Minor.LEFT;
    }

    public static String literal() {
        return "dt13.NestedShape$Major";
    }
}
