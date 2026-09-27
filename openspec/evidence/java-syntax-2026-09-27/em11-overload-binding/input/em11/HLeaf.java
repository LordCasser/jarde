package em11;

import java.util.ArrayList;

public class HLeaf extends HMid {
    public static int created;

    public HLeaf() { created++; }

    public String call(ArrayList<String> value) { return "leaf-ArrayList"; }
}
