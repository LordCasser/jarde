package cf11;

import java.util.List;

public class DistinctStringFixture {
    public static int effects;

    public void renamed(List<String> chosen, List<String> offered) {
        for (String candidate : chosen) {
            for (String option : offered) {
                if (candidate.equals(option)) {
                    if (candidate.length() == 5) {
                        offered.add(candidate);
                        effects++;
                    } else {
                        chosen.remove((Object) option);
                        effects++;
                    }
                }
            }
        }
        if (offered.size() > 0) {
            chosen.clear();
            effects++;
        }
    }
}
