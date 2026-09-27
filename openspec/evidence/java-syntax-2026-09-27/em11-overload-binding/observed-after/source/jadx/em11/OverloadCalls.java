package em11;

import java.util.ArrayList;
import java.util.List;

/* JADX INFO: loaded from: input.jar:em11/OverloadCalls.class */
public class OverloadCalls {
    public static String call(String str) {
        return "String";
    }

    public static String call(List<String> list) {
        return "List";
    }

    public static String call(ArrayList<String> arrayList) {
        return "ArrayList";
    }

    public static String choose(Object[][] objArr, Object obj) {
        return "Object[][]";
    }

    public static String choose(int[][] iArr, int i) {
        return "int[][]";
    }

    public static String run(Object obj) {
        int[][][] iArr = new int[1][1][1];
        return call((ArrayList<String>) new ArrayList()) + "/" + call((List<String>) new ArrayList()) + "/" + call((String) null) + "/" + call((List<String>) null) + "/" + call((ArrayList<String>) null) + "/" + (obj instanceof String ? call((String) obj) : "none") + "/" + choose((Object[][]) iArr, (Object) (-1)) + "/" + choose(iArr[0], -2);
    }
}
