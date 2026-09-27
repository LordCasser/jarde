package em11;

import java.util.ArrayList;
import java.util.List;

public class OverloadCalls {
    public static String call(String value) { return "String"; }
    public static String call(List<String> value) { return "List"; }
    public static String call(ArrayList<String> value) { return "ArrayList"; }

    public static String choose(Object[][] value, Object marker) { return "Object[][]"; }
    public static String choose(int[][] value, int marker) { return "int[][]"; }

    public static String run(Object value) {
        String first = call(new ArrayList<String>());
        String second = call((List<String>) new ArrayList<String>());
        String third = call((String) null);
        String fourth = call((List<String>) null);
        String fifth = call((ArrayList<String>) null);
        String sixth = value instanceof String ? call((String) value) : "none";
        int[][][] cube = new int[1][1][1];
        String seventh = choose(cube, -1);
        String eighth = choose(cube[0], -2);
        return first + "/" + second + "/" + third + "/" + fourth + "/"
                + fifth + "/" + sixth + "/" + seventh + "/" + eighth;
    }
}
