package cf11;

import java.util.List;
import java.util.Iterator;

public class OrdinaryTryListFixture {
    public static int effects;

    // Exact ordinary-try branch topology from baseline.tryStringBranch, with only source names
    // changed. This is the independent rename-admission control.
    public void renamedTry(List<String> source, List<String> sink) {
        try {
            for (String element : source) {
                if (element == null) {
                    sink.add(null);
                } else if (element.length() == 0) {
                    sink.add(element);
                } else {
                    sink.add(null);
                }
                effects++;
            }
        } catch (RuntimeException problem) {
            effects++;
        }
    }

    // Same proven branch topology with a direct catch-owned List use admitted by the finite
    // generic-use contract.
    public void stringBranch(List<String> input, List<String> output) {
        try {
            for (String value : input) {
                if (value == null) {
                    output.add(null);
                } else if (value.length() == 0) {
                    output.add(value);
                } else {
                    output.add(null);
                }
                effects++;
            }
        } catch (RuntimeException failure) {
            output.add(null);
            effects++;
        }
    }

    // Exact ordinary-try branch topology from baseline.tryIntegerBranch.
    public void integerTry(List<Integer> selected, List<Integer> available) {
        try {
            for (Integer value : selected) {
                if (value == null) {
                    available.add(null);
                } else if (value.intValue() > 0) {
                    available.add(value);
                } else {
                    available.add(null);
                }
                effects++;
            }
        } catch (RuntimeException failure) {
            effects++;
        }
    }

    // Exact nested-foreach topology from baseline.tryStringNested, with source names changed.
    public void nestedTry(List<String> source, List<String> witness, List<String> destination) {
        try {
            for (String outer : source) {
                for (String inner : witness) {
                    if (outer.equals(inner)) {
                        destination.add(outer);
                    }
                }
                effects++;
            }
        } catch (RuntimeException failure) {
            effects++;
        }
    }

    // Exact ordinary-prefix-plus-loop topology from baseline.tryStringRemovePrefix.
    public void stringRemovePrefix(List<String> input, List<String> output, Object key) {
        try {
            output.remove((Object) key);
            for (String value : input) {
                output.add(value);
                effects++;
            }
        } catch (RuntimeException failure) {
            effects++;
        }
    }

    // The explicit iterator initializer and the marker write are both in the protected
    // region, before the first loop test. If iterator() throws, catch observes marker == 0;
    // this is a negative control for the iterator-to-loop gap write proof.
    public void catchVisibleIteratorGap(List<String> input, List<String> output) {
        int marker = 0;
        try {
            Iterator<String> cursor = input.iterator();
            marker = 1;
            while (cursor.hasNext()) {
                output.add(cursor.next());
                effects++;
            }
        } catch (RuntimeException failure) {
            effects = marker;
        }
    }

    public void unknownCatchUse(List<String> input, List<String> output) {
        try {
            for (String value : input) {
                output.add(value);
            }
        } catch (RuntimeException failure) {
            consume(output);
        }
    }

    private static void consume(Object value) {
        effects += value == null ? 0 : 1;
    }

    public void reassignedInCatch(List<String> input, List<String> replacement) {
        try {
            for (String value : input) {
                replacement.add(value);
            }
        } catch (RuntimeException failure) {
            input = replacement;
            input.add(null);
        }
    }

    @SuppressWarnings({"rawtypes", "unchecked"})
    public void rawAliasInCatch(List<String> input, List<String> output) {
        try {
            for (String value : input) {
                output.add(value);
            }
        } catch (RuntimeException failure) {
            List escaped = output;
            escaped.add("polluted");
        }
    }

    public void multiCatch(List<String> input, List<String> output) {
        try {
            for (String value : input) {
                output.add(value);
            }
        } catch (IllegalArgumentException | IllegalStateException failure) {
            output.add(null);
        }
    }

    public void indexOverload(List<Integer> values, List<Integer> witness, Integer key, int position) {
        try {
            for (Integer value : values) {
                values.remove((Object) key);
                witness.add(value);
                values.remove(position);
            }
        } catch (RuntimeException failure) {
            witness.add(null);
        }
    }
}
