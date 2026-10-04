package defpackage;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.HashMap;
import java.util.Iterator;
import java.util.List;
import java.util.Map;

/* JADX INFO: loaded from: S4.class */
public class S4 {
    public static List<String> nestedNoCast(List<String> list, List<String> list2) {
        ArrayList arrayList = new ArrayList();
        for (String str : list) {
            Iterator<String> it = list2.iterator();
            while (it.hasNext()) {
                arrayList.add(str + ":" + it.next());
            }
        }
        return arrayList;
    }

    public static List<String> singleCastConcat(List<Object> list) {
        ArrayList arrayList = new ArrayList();
        Iterator<Object> it = list.iterator();
        while (it.hasNext()) {
            arrayList.add(((String) it.next()) + "!");
        }
        return arrayList;
    }

    public static List<String> nestedPreStored(Map<String, List<Integer>> map, String str) {
        ArrayList arrayList = new ArrayList();
        for (Map.Entry<String, List<Integer>> entry : map.entrySet()) {
            String key = entry.getKey();
            if (key.startsWith(str)) {
                Iterator<Integer> it = entry.getValue().iterator();
                while (it.hasNext()) {
                    arrayList.add(key + ":" + it.next().intValue());
                }
            }
        }
        return arrayList;
    }

    public static void main(String[] strArr) {
        System.out.println(nestedNoCast(Arrays.asList("k"), Arrays.asList("v")));
        System.out.println(singleCastConcat(Arrays.asList("c")));
        HashMap map = new HashMap();
        map.put("px", Arrays.asList(3, 5));
        System.out.println(nestedPreStored(map, "p"));
    }
}
