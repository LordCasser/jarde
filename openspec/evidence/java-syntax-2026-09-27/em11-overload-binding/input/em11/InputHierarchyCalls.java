package em11;

public class InputHierarchyCalls {
    public static String take(HMid value) { return "mid"; }
    public static String take(HLeaf value) { return "leaf"; }

    public static String run() {
        return take((HMid) new HLeaf());
    }
}
