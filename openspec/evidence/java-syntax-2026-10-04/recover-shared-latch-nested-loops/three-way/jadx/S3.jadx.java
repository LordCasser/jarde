package defpackage;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;

/* JADX INFO: loaded from: S3.class */
public class S3 {
    public static List<String> nestedBreak(Map<String, List<Integer>> map, String str) {
        ArrayList arrayList = new ArrayList();
        for (Map.Entry<String, List<Integer>> entry : map.entrySet()) {
            if (entry.getKey().startsWith(str)) {
                for (Integer num : entry.getValue()) {
                    if (num.intValue() < 0) {
                        break;
                    }
                    arrayList.add(entry.getKey() + ":" + num);
                }
            }
        }
        return arrayList;
    }

    public static List<String> nestedNoJump(Map<String, List<Integer>> map, String str) {
        ArrayList arrayList = new ArrayList();
        for (Map.Entry<String, List<Integer>> entry : map.entrySet()) {
            if (entry.getKey().startsWith(str)) {
                Iterator<Integer> it = entry.getValue().iterator();
                while (it.hasNext()) {
                    arrayList.add(entry.getKey() + ":" + it.next());
                }
            }
        }
        return arrayList;
    }

    public static List<String> singleLoopConcat(List<String> list) {
        ArrayList arrayList = new ArrayList();
        Iterator<String> it = list.iterator();
        while (it.hasNext()) {
            arrayList.add(it.next() + "!");
        }
        return arrayList;
    }

    public static void main(String[] strArr) {
        HashMap map = new HashMap();
        map.put("px", Arrays.asList(3, -1, 5));
        System.out.println(nestedBreak(map, "p"));
        System.out.println(nestedNoJump(map, "p"));
        System.out.println(singleLoopConcat(Arrays.asList("a", "b")));
    }
}
