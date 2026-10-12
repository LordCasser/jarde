package cf11;

import java.util.Iterator;
import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11OuterAssignmentTarget.class */
public final class CF11OuterAssignmentTarget {
    public static String positive(List list) {
        String str = "seed";
        Iterator it = list.iterator();
        while (it.hasNext()) {
            str = (String) it.next();
            Runner.assigned(str);
        }
        return str;
    }

    public static Object noCastAssign(List list) {
        Object next = "seed";
        Iterator it = list.iterator();
        while (it.hasNext()) {
            next = it.next();
        }
        return next;
    }

    public static String bodyEffectBeforeAssign(List list) {
        String str = "seed";
        Iterator it = list.iterator();
        while (it.hasNext()) {
            Runner.beforeAssignment();
            str = (String) it.next();
            Runner.assigned(str);
        }
        return str;
    }

    public static String extraNextConsumption(List list) {
        String str = "seed";
        Iterator it = list.iterator();
        while (it.hasNext()) {
            it.next();
            str = (String) it.next();
            Runner.assigned(str);
        }
        return str;
    }
}
