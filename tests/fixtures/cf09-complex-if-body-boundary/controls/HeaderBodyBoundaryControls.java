import java.util.Map;

/** Root-authored IR controls; these are not a replacement for the complete upstream case. */
public final class HeaderBodyBoundaryControls {
    public static int andPrefix(Map<String, Integer> map, int x, int limit) {
        int length = 0;
        while (x < 100 && x < limit) {
            Integer tile = map.get(x + "");
            if (tile == null || tile.intValue() != 100) break;
            length++;
            x++;
        }
        return length;
    }

    public static int orBody(Map<String, Integer> map, int x, int limit) {
        int length = 0;
        while (x < 100 || x < limit) {
            Integer tile = map.get(x + "");
            if (tile == null || tile.intValue() != 100) break;
            length++;
            x++;
        }
        return length;
    }

    public static int initialEffect(Map<String, Integer> map, int x, int limit) {
        int length = 0;
        while ((x = map.size()) < 100 && length < limit) {
            Integer tile = map.get(x + "");
            if (tile == null || tile.intValue() != 100) break;
            length++;
        }
        return length;
    }
    public static int initialStatements(Map<String, Integer> map, int x, int limit) {
        int length = 0;
        while (true) {
            map.put("seen", x);
            if (x >= 100 || length >= limit) break;
            Integer tile = map.get(x + "");
            if (tile == null || tile.intValue() != 100) break;
            length++;
        }
        return length;
    }
}
