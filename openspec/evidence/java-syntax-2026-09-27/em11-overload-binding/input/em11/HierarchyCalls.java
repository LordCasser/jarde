package em11;

import java.util.ArrayList;
import java.util.List;

public class HierarchyCalls {
    public static String run() {
        HLeaf leaf = new HLeaf();
        return leaf.call(new ArrayList<String>()) + "/"
                + leaf.call((List<String>) new ArrayList<String>()) + "/"
                + leaf.call((String) null) + "/"
                + leaf.call((List<String>) null) + "/"
                + leaf.call((ArrayList<String>) null) + "/"
                + ((HBase) leaf).call(null) + "/"
                + ((HMid) leaf).call((List<String>) null);
    }
}
