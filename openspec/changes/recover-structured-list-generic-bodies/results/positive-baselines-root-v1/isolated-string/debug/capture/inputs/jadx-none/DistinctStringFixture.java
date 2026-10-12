package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/DistinctStringFixture.class */
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
                        chosen.remove(option);
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
