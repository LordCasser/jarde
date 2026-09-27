package em11;

import java.util.ArrayList;

/* JADX INFO: loaded from: input.jar:em11/HLeaf.class */
public class HLeaf extends HMid {
    public static int created;

    public HLeaf() {
        created++;
    }

    public String call(ArrayList<String> arrayList) {
        return "leaf-ArrayList";
    }
}
