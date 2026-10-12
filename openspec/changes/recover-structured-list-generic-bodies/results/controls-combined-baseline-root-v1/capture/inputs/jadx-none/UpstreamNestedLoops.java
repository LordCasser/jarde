package cf11;

import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/UpstreamNestedLoops.class */
public class UpstreamNestedLoops {
    public static int effects;

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

    public void renamed(List<String> chosen, List<String> offered) {
        for (String candidate : chosen) {
            for (String option : offered) {
                if (candidate.equals(option)) {
                    if (candidate.length() == 5) {
                        offered.add(candidate);
                    } else {
                        chosen.remove(option);
                    }
                }
            }
        }
        if (offered.size() > 0) {
            chosen.clear();
        }
    }

    public void numbers(List<Integer> selected, List<Integer> available) {
        for (Integer value : selected) {
            for (Integer option : available) {
                if (value.equals(option)) {
                    if (value.intValue() == 5) {
                        available.add(value);
                    } else {
                        selected.remove(option);
                    }
                }
            }
        }
        if (available.size() > 0) {
            selected.clear();
        }
    }

    public void castBoundary(List<String> source, List<String> destination) {
        for (Object rawValue : source) {
            String checkedValue = (String) rawValue;
            for (Object rawProbe : destination) {
                String checkedProbe = (String) rawProbe;
                if (checkedValue.equals(checkedProbe)) {
                    destination.add(checkedValue);
                    effects++;
                }
            }
        }
    }

    public void rawLists(List left, List right) {
        for (Object value : left) {
            for (Object option : right) {
                if (value.equals(option)) {
                    right.add(value);
                    left.remove(option);
                    left.remove(0);
                    effects++;
                }
            }
        }
    }

    public void rawAliasEscape(List<String> target, List<Integer> source) {
        for (Integer value : source) {
            target.add(value);
        }
        for (String str : target) {
            effects++;
        }
    }

    public void reassigned(List<String> input, List<String> replacement) {
        for (String value : replacement) {
            replacement.add(value);
            effects++;
        }
    }

    public void wildcardNull(List<? extends Number> values, List<Number> sink) {
        for (Number value : values) {
            values.add(null);
            sink.add(value);
            effects++;
        }
    }

    public void handled(List<String> input, List<String> output) {
        try {
            for (String value : input) {
                output.add(value);
                effects++;
            }
        } catch (RuntimeException e) {
            effects++;
        }
    }

    public void indexOverload(List<Integer> values, List<Integer> witness, Integer key, int position) {
        values.remove(key);
        values.remove(position);
        witness.add(key);
        effects++;
    }
}
