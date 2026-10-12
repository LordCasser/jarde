package cf11;

import java.util.Iterator;
import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/UpstreamNestedLoops.class */
public class UpstreamNestedLoops {
    public static int effects;

    public void test(List<String> list, List<String> list2) {
        for (String str : list) {
            for (String str2 : list2) {
                if (str.equals(str2)) {
                    if (str.length() == 5) {
                        list2.add(str);
                    } else {
                        list.remove(str2);
                    }
                }
            }
        }
        if (list2.size() > 0) {
            list.clear();
        }
    }

    public void renamed(List<String> list, List<String> list2) {
        for (String str : list) {
            for (String str2 : list2) {
                if (str.equals(str2)) {
                    if (str.length() == 5) {
                        list2.add(str);
                    } else {
                        list.remove(str2);
                    }
                }
            }
        }
        if (list2.size() > 0) {
            list.clear();
        }
    }

    public void numbers(List<Integer> list, List<Integer> list2) {
        for (Integer num : list) {
            for (Integer num2 : list2) {
                if (num.equals(num2)) {
                    if (num.intValue() == 5) {
                        list2.add(num);
                    } else {
                        list.remove(num2);
                    }
                }
            }
        }
        if (list2.size() > 0) {
            list.clear();
        }
    }

    public void castBoundary(List<String> list, List<String> list2) {
        for (String str : list) {
            Iterator<String> it = list2.iterator();
            while (it.hasNext()) {
                if (str.equals(it.next())) {
                    list2.add(str);
                    effects++;
                }
            }
        }
    }

    public void rawLists(List list, List list2) {
        for (Object obj : list) {
            for (Object obj2 : list2) {
                if (obj.equals(obj2)) {
                    list2.add(obj);
                    list.remove(obj2);
                    list.remove(0);
                    effects++;
                }
            }
        }
    }

    public void rawAliasEscape(List<String> list, List<Integer> list2) {
        Iterator<Integer> it = list2.iterator();
        while (it.hasNext()) {
            list.add(it.next());
        }
        for (String str : list) {
            effects++;
        }
    }

    public void reassigned(List<String> list, List<String> list2) {
        Iterator<String> it = list2.iterator();
        while (it.hasNext()) {
            list2.add(it.next());
            effects++;
        }
    }

    public void wildcardNull(List<? extends Number> list, List<Number> list2) {
        for (Number number : list) {
            list.add(null);
            list2.add(number);
            effects++;
        }
    }

    public void handled(List<String> list, List<String> list2) {
        try {
            Iterator<String> it = list.iterator();
            while (it.hasNext()) {
                list2.add(it.next());
                effects++;
            }
        } catch (RuntimeException e) {
            effects++;
        }
    }

    public void indexOverload(List<Integer> list, List<Integer> list2, Integer num, int i) {
        list.remove(num);
        list.remove(i);
        list2.add(num);
        effects++;
    }
}
