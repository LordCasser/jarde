package defpackage;

import java.io.IOException;
import java.util.List;

/* JADX INFO: loaded from: complex-method.class */
public class ComplexMethodProbe {
    public static <T extends Number> List<? extends T>[] choose(List<? extends T>[] listArr) throws IOException {
        if (listArr.length == 0) {
            throw new IOException();
        }
        return listArr;
    }
}
