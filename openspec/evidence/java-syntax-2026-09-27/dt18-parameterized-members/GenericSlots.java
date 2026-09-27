package dt18;

import java.util.List;

public class GenericSlots {
    public List<String> names;
    public List raw;

    public List<String> id(List<String> input) {
        return input;
    }

    public List<String> empty() {
        return null;
    }

    public List raw(List input) {
        return input;
    }
}
