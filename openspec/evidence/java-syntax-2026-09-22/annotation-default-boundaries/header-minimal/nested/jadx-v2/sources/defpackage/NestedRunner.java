package defpackage;

/* JADX INFO: loaded from: NestedRunner.class */
class NestedRunner {
    NestedRunner() {
    }

    public static void main(String[] strArr) throws Exception {
        System.out.println(((Inner) Nested.class.getMethod("child", new Class[0]).getDefaultValue()).value());
        System.out.println(((Inner[]) Nested.class.getMethod("children", new Class[0]).getDefaultValue()).length);
    }
}
