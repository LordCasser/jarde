package cf11;

import java.util.List;

public class DistinctIntegerFixture {
    public static int effects;

    public void numbers(List<Integer> selected, List<Integer> available) {
        for (Integer value : selected) {
            for (Integer option : available) {
                if (value.equals(option)) {
                    if (value.intValue() == 5) {
                        available.add(option);
                        effects++;
                    } else {
                        selected.remove((Object) option);
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
