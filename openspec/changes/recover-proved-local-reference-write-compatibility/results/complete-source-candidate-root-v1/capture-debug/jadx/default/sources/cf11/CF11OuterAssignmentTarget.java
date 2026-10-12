package cf11;

import java.util.Iterator;
import java.util.List;

/* JADX INFO: loaded from: for-condition-controls.jar:cf11/CF11OuterAssignmentTarget.class */
public final class CF11OuterAssignmentTarget {
    public static String positive(List values) {
        String target = "seed";
        Iterator iterator = values.iterator();
        while (iterator.hasNext()) {
            target = (String) iterator.next();
            Runner.assigned(target);
        }
        return target;
    }

    public static Object noCastAssign(List values) {
        Object target = "seed";
        Iterator iterator = values.iterator();
        while (iterator.hasNext()) {
            target = iterator.next();
        }
        return target;
    }

    public static String bodyEffectBeforeAssign(List values) {
        String target = "seed";
        Iterator iterator = values.iterator();
        while (iterator.hasNext()) {
            Runner.beforeAssignment();
            target = (String) iterator.next();
            Runner.assigned(target);
        }
        return target;
    }

    public static String extraNextConsumption(List values) {
        String target = "seed";
        Iterator iterator = values.iterator();
        while (iterator.hasNext()) {
            iterator.next();
            target = (String) iterator.next();
            Runner.assigned(target);
        }
        return target;
    }
}
