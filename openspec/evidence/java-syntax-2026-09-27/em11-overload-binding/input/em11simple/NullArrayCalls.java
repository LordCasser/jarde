package em11simple;

import java.util.ArrayList;
import java.util.List;

public class NullArrayCalls {
    public static String call(String value) { return "String"; }
    public static String call(List<String> value) { return "List"; }
    public static String call(ArrayList<String> value) { return "ArrayList"; }
    public static String choose(Object[][] value, Object marker) { return "Object[][]"; }
    public static String choose(int[][] value, int marker) { return "int[][]"; }

    public static String run() {
        int[][][] cube = new int[1][1][1];
        return call((String) null) + "/" + call((List<String>) null) + "/"
                + call((ArrayList<String>) null) + "/" + choose(cube, -1) + "/"
                + choose(cube[0], -2);
    }
}
