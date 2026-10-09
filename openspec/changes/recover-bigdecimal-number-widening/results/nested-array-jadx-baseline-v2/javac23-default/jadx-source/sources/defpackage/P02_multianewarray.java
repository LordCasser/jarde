package defpackage;

import java.util.Arrays;
import java.util.List;

/* JADX INFO: loaded from: P02_multianewarray.class */
public class P02_multianewarray {
    static int sum(List<Integer> list) {
        int[][] iArr = {new int[]{0}};
        list.forEach(num -> {
            int[] iArr2 = iArr[0];
            iArr2[0] = iArr2[0] + num.intValue();
        });
        return iArr[0][0];
    }

    public static void main(String[] strArr) {
        System.out.println(sum(Arrays.asList(1, 2, 3)));
    }
}
