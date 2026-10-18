package jadx.tests.integration.loops;

import java.util.ArrayList;
import java.util.List;

/** Separate complete-source probes; no recovery result is assumed. */
public final class NestedLoops2GenericControls {
    private static List<?> escaped;

    private NestedLoops2GenericControls() { }

    public static void voidGetCastString(List<String> values) {
        for (int i = 0; i < values.size(); i++) {
            String value = (String) values.get(i);
            value.length();
        }
    }

    public static boolean fixedBoundBoolean(List<String> values) {
        boolean seen = false;
        for (int i = 0; i < 2; i++) {
            String value = values.get(i);
            seen |= value.length() > 0;
        }
        return seen;
    }

    public static int primitiveInt(List<String> values) {
        int result = 0;
        for (int i = 0; i < values.size(); i++) result += values.get(i).length();
        return result;
    }

    public static long primitiveLong(List<String> values) {
        long result = 0L;
        for (int i = 0; i < values.size(); i++) result += values.get(i).length();
        return result;
    }

    public static int legalWrongCast(List<String> values) {
        int result = 0;
        for (int i = 0; i < values.size(); i++) {
            Integer value = (Integer) (Object) values.get(i);
            result += value.intValue();
        }
        return result;
    }

    public static int reassignAndEscape(List<String> values) {
        List<String> alias = values;
        escaped = alias;
        values = new ArrayList<String>(values);
        int result = 0;
        for (int i = 0; i < values.size(); i++) result += values.get(i).length();
        return result;
    }

    @SuppressWarnings("rawtypes")
    public static int rawList(List values) {
        int result = 0;
        for (int i = 0; i < values.size(); i++) result += String.valueOf(values.get(i)).length();
        return result;
    }

    public static int wildcardList(List<?> values) {
        int result = 0;
        for (int i = 0; i < values.size(); i++) result += String.valueOf(values.get(i)).length();
        return result;
    }

    public static Object referenceResult(List<String> values) {
        return values.get(0);
    }
}
