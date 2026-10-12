package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/DistinctIntegerFixture.class */
public class DistinctIntegerFixture {
    public static int effects;

    public void numbers(List<Integer> selected, List<Integer> available) {
        for (Integer value : selected) {
            for (Integer option : available) {
                if (value.equals(option)) {
                    if (value.intValue() == 5) {
                        available.add(value);
                        effects++;
                    } else {
                        selected.remove(option);
                        effects++;
                    }
                }
            }
        }
        if (available.size() > 0) {
            selected.clear();
            effects++;
        }
    }
}
