package defpackage;

import java.lang.Comparable;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/* JADX INFO: loaded from: SCGB.jar:SCGB.class */
public class SCGB<T extends Comparable<T>> {
    private Map<String, List<T>> index = new HashMap();

    public static void main(String[] strArr) {
        SCGB scgb = new SCGB();
        System.out.println("index:" + (scgb.index != null));
        System.out.println("read:" + (scgb.index instanceof Map));
    }
}
