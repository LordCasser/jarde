package em11simple;

import java.util.ArrayList;
import java.util.List;

/* JADX INFO: loaded from: simple.jar:em11simple/NullArrayCalls.class */
public class NullArrayCalls {
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

    public static String run() {
        int[][][] iArr = new int[1][1][1];
        return call((String) null) + "/" + call((List<String>) null) + "/" + call((ArrayList<String>) null) + "/" + choose((Object[][]) iArr, (Object) (-1)) + "/" + choose(iArr[0], -2);
    }
}
