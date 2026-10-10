package defpackage;

import java.util.Iterator;
import java.util.List;

/* JADX INFO: loaded from: input.jar:VariablePostfixLoop.class */
public class VariablePostfixLoop {
    public static int countEmpty(List<String> list) {
        int i = 0;
        if (list != null) {
            Iterator<String> it = list.iterator();
            while (it.hasNext()) {
                if (it.next().isEmpty()) {
                    i++;
                }
            }
        }
        return i;
    }
}
