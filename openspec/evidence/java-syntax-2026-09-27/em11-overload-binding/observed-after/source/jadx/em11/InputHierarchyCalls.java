package em11;

/* JADX INFO: loaded from: input.jar:em11/InputHierarchyCalls.class */
public class InputHierarchyCalls {
    public static String take(HMid hMid) {
        return "mid";
    }

    public static String take(HLeaf hLeaf) {
        return "leaf";
    }

    public static String run() {
        return take((HMid) new HLeaf());
    }
}
