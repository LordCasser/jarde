package dt13;

/* JADX INFO: loaded from: input.jar:dt13/NestedShape.class */
public final class NestedShape {

    /* JADX INFO: loaded from: input.jar:dt13/NestedShape$Major.class */
    public enum Major {
        FIRST,
        SECOND;

        /* JADX INFO: loaded from: input.jar:dt13/NestedShape$Major$Minor.class */
        public enum Minor {
            LEFT,
            RIGHT
        }
    }

    public static String observe() {
        return Major.FIRST + ":" + Major.Minor.LEFT;
    }

    public static String literal() {
        return "dt13.NestedShape$Major";
    }
}
