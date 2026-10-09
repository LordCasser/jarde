package defpackage;

import java.lang.Comparable;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/* JADX INFO: loaded from: SCGBCompat.jar:SCGBCompat.class */
public class SCGBCompat<T extends Comparable<T>> {
    private Map<String, List<T>> index = new HashMap();

    public static void main(String[] strArr) {
        SCGBCompat sCGBCompat = new SCGBCompat();
        if (sCGBCompat.index != null) {
            System.out.println("index:true");
        } else {
            System.out.println("index:false");
        }
        if (sCGBCompat.index instanceof Map) {
            System.out.println("read:true");
        } else {
            System.out.println("read:false");
        }
    }
}
