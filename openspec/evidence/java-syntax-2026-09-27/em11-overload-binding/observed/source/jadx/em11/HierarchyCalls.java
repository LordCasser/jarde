package em11;

import java.util.ArrayList;
import java.util.List;

/* JADX INFO: loaded from: input.jar:em11/HierarchyCalls.class */
public class HierarchyCalls {
    public static String run() {
        HLeaf hLeaf = new HLeaf();
        return hLeaf.call(new ArrayList<>()) + "/" + hLeaf.call((List<String>) new ArrayList()) + "/" + hLeaf.call((String) null) + "/" + hLeaf.call((List<String>) null) + "/" + hLeaf.call((ArrayList<String>) null) + "/" + hLeaf.call((String) null) + "/" + hLeaf.call((List<String>) null);
    }
}
