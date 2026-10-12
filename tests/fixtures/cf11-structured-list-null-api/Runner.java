package cf11;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

public class Runner {
    private static ArrayList<String> strings(String... values) {
        return new ArrayList<>(Arrays.asList(values));
    }

    private static ArrayList<Integer> integers(Integer... values) {
        return new ArrayList<>(Arrays.asList(values));
    }

    private static String state(Object value) {
        return String.valueOf(value);
    }

    private static void runString(String name, List<String> selected, List<String> available) {
        NullStringListFixture.effects = 0;
        Throwable failure = null;
        try {
            new NullStringListFixture().strings(selected, available);
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(name
                + "|exception=" + (failure == null ? "none" : failure.getClass().getName())
                + "|selected=" + state(selected)
                + "|available=" + state(available)
                + "|effects=" + NullStringListFixture.effects);
    }

    private static void runInteger(String name, List<Integer> selected, List<Integer> available) {
        NullIntegerListFixture.effects = 0;
        Throwable failure = null;
        try {
            new NullIntegerListFixture().integers(selected, available);
        } catch (Throwable error) {
            failure = error;
        }
        System.out.println(name
                + "|exception=" + (failure == null ? "none" : failure.getClass().getName())
                + "|selected=" + state(selected)
                + "|available=" + state(available)
                + "|effects=" + NullIntegerListFixture.effects);
    }

    public static void main(String[] args) {
        runString("string-null-add-cme", strings("alpha"), strings("alpha"));
        runString("string-no-match", strings("other"), strings("alpha"));
        runInteger("integer-null-add-cme", integers(5), integers(5));
        runInteger("integer-no-match", integers(8), integers(5));
    }
}
