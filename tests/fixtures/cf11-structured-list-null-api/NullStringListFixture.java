package cf11;

import java.util.List;

public class NullStringListFixture {
    public static int effects;

    public void strings(List<String> selected, List<String> available) {
        for (String value : selected) {
            for (String option : available) {
                if (value.equals(option)) {
                    if (value.length() == 5) {
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
