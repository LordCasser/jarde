package cf11;

import java.util.List;

public class NullIntegerListFixture {
    public static int effects;

    public void integers(List<Integer> selected, List<Integer> available) {
        for (Integer value : selected) {
            for (Integer option : available) {
                if (value.equals(option)) {
                    if (value.intValue() == 5) {
                        available.add(null);
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
