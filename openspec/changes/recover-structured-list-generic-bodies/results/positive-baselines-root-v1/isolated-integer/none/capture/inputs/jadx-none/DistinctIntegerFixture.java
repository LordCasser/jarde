package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/DistinctIntegerFixture.class */
public class DistinctIntegerFixture {
    public static int effects;

    public void numbers(List<Integer> list, List<Integer> list2) {
        for (Integer num : list) {
            for (Integer num2 : list2) {
                if (num.equals(num2)) {
                    if (num.intValue() == 5) {
                        list2.add(num);
                        effects++;
                    } else {
                        list.remove(num2);
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
