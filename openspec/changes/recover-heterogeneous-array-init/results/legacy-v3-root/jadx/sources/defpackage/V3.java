package defpackage;

import java.util.ArrayList;
import java.util.Arrays;

/* JADX INFO: loaded from: input.jar:V3.class */
public class V3 {
    public static int viaMixed() {
        return new ArrayList(Arrays.asList(1, 2L, Double.valueOf(3.0d))).size();
    }

    public static void main(String[] strArr) {
        System.out.println(viaMixed());
    }
}
