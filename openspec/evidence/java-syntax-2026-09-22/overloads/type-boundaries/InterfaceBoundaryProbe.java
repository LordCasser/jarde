public class InterfaceBoundaryProbe {
    public static int take(Runnable value) {
        return 7;
    }

    public static int caller(Object value) {
        return take((Runnable) value);
    }
}
