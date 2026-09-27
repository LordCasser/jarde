package dt14;

import java.util.HashMap;
import java.util.Map;

public enum CustomInit {
    RED,
    BLUE;

    public static final Map<String, CustomInit> BY_NAME = new HashMap<>();

    static {
        for (CustomInit value : values()) {
            BY_NAME.put(value.name(), value);
        }
    }
}
