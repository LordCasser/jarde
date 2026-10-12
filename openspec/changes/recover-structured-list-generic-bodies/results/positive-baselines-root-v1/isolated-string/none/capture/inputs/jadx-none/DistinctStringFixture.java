package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/DistinctStringFixture.class */
public class DistinctStringFixture {
    public static int effects;

    public void renamed(List<String> list, List<String> list2) {
        for (String str : list) {
            for (String str2 : list2) {
                if (str.equals(str2)) {
                    if (str.length() == 5) {
                        list2.add(str);
                        effects++;
                    } else {
                        list.remove(str2);
                        effects++;
                    }
                }
            }
        }
        if (list2.size() > 0) {
            list.clear();
            effects++;
        }
    }
}
