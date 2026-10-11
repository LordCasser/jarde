package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/UpstreamNestedLoops.class */
public class UpstreamNestedLoops {
    public void test(List<String> l1, List<String> l2) {
        for (String s1 : l1) {
            for (String s2 : l2) {
                if (s1.equals(s2)) {
                    if (s1.length() == 5) {
                        l2.add(s1);
                    } else {
                        l1.remove(s2);
                    }
                }
            }
        }
        if (l2.size() > 0) {
            l1.clear();
        }
    }
}
