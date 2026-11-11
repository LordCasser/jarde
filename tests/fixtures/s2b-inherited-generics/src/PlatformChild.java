import java.util.ArrayList;
import java.util.List;

public class PlatformChild extends ArrayList<String> {
    public List<String> names;

    public String inheritedRead(int index) {
        return get(index);
    }

    public String returnInherited() {
        return get(0);
    }

    public List<String> listRead() {
        return names;
    }
}
