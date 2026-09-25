package defpackage;

import java.util.Arrays;

/* JADX INFO: loaded from: BasicRunner.class */
class BasicRunner {
    BasicRunner() {
    }

    public static void main(String[] strArr) throws Exception {
        System.out.println(Basic.class.getMethod("count", new Class[0]).getDefaultValue());
        System.out.println(Basic.class.getMethod("label", new Class[0]).getDefaultValue());
        System.out.println(Arrays.toString((int[]) Basic.class.getMethod("codes", new Class[0]).getDefaultValue()));
    }
}
