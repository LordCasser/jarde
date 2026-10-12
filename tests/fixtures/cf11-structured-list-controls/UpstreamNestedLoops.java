package cf11;

import java.util.List;

public class UpstreamNestedLoops {
    public static int effects;

    // Frozen CF11 method body: the fixed upstream List<String> shape.
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

    // Same supported body with distinct class/method/parameter/local names.
    public void renamed(List<String> chosen, List<String> offered) {
        for (String candidate : chosen) {
            for (String option : offered) {
                if (candidate.equals(option)) {
                    if (candidate.length() == 5) {
                        offered.add(candidate);
                    } else {
                        chosen.remove((Object) option);
                    }
                }
            }
        }
        if (offered.size() > 0) {
            chosen.clear();
        }
    }

    // Different exact element type; Object overload is intentional.
    public void numbers(List<Integer> selected, List<Integer> available) {
        for (Integer value : selected) {
            for (Integer option : available) {
                if (value.equals(option)) {
                    if (value.intValue() == 5) {
                        available.add(value);
                    } else {
                        selected.remove((Object) option);
                    }
                }
            }
        }
        if (available.size() > 0) {
            selected.clear();
        }
    }

    // Explicit Object widening plus real String casts from Object locals.
    public void castBoundary(List<String> source, List<String> destination) {
        for (Object rawValue : source) {
            String checkedValue = (String) (Object) rawValue;
            for (Object rawProbe : destination) {
                String checkedProbe = (String) (Object) rawProbe;
                if (checkedValue.equals((Object) checkedProbe)) {
                    destination.add(checkedValue);
                    effects++;
                }
            }
        }
    }

    // Raw parameters are a raw-source control, not a generic refusal case.
    public void rawLists(List left, List right) {
        for (Object value : left) {
            for (Object option : right) {
                if (value.equals(option)) {
                    right.add(value);
                    left.remove((Object) option);
                    left.remove(0);
                    effects++;
                }
            }
        }
    }

    // Legal Java heap pollution through a raw alias; a subsequent String read casts.
    @SuppressWarnings({"rawtypes", "unchecked"})
    public void rawAliasEscape(List<String> target, List<Integer> source) {
        List escaped = target;
        for (Integer value : source) {
            escaped.add(value);
        }
        for (String value : target) {
            effects++;
        }
    }

    // Parameter reassignment is legal and intentionally outside the projection proof.
    public void reassigned(List<String> input, List<String> replacement) {
        input = replacement;
        for (String value : input) {
            replacement.add(value);
            effects++;
        }
    }

    // Null is legal for the wildcard capture, while wildcard writes remain unsupported.
    public void wildcardNull(List<? extends Number> values, List<Number> sink) {
        for (Number value : values) {
            values.add(null);
            sink.add(value);
            effects++;
        }
    }

    // Handler-bearing structured body is deliberately outside the generic proof.
    public void handled(List<String> input, List<String> output) {
        try {
            for (String value : input) {
                output.add(value);
                effects++;
            }
        } catch (RuntimeException failure) {
            effects++;
        }
    }

    // Descriptor control: remove(Object) then remove(int); exact call target matters.
    public void indexOverload(List<Integer> values, List<Integer> witness, Integer key, int position) {
        values.remove((Object) key);
        values.remove(position);
        witness.add(key);
        effects++;
    }
}
