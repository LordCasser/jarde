package defpackage;

/* JADX INFO: loaded from: fixture.jar:FunctionalConstructors.class */
public class FunctionalConstructors {
    public static int trace;

    public static java.lang.Thread runnable() {
        return new java.lang.Thread(() -> {
            trace += 3;
        });
    }

    public static java.lang.Thread captured(int i) {
        return new java.lang.Thread(() -> {
            trace += i;
        });
    }

    public static java.util.PriorityQueue<java.lang.Integer> comparator() {
        return new java.util.PriorityQueue<>((num, num2) -> {
            return num2.intValue() - num.intValue();
        });
    }

    public static java.util.PriorityQueue<java.lang.Integer> reference() {
        return new java.util.PriorityQueue<>(defpackage.FunctionalConstructors::compare);
    }

    public static int compare(java.lang.Integer num, java.lang.Integer num2) {
        return num2.intValue() - num.intValue();
    }

    public static java.util.concurrent.FutureTask<java.lang.Integer> callable() {
        return new java.util.concurrent.FutureTask<>(() -> {
            return 7;
        });
    }

    public static defpackage.IntBox primitive(int i) {
        return new defpackage.IntBox(i2 -> {
            return i + i2;
        });
    }

    public static defpackage.IntBox primitiveReference() {
        return new defpackage.IntBox(defpackage.FunctionalConstructors::twice);
    }

    public static int twice(int i) {
        return i * 2;
    }

    public static java.lang.String name() {
        trace = (trace * 10) + 1;
        return "worker";
    }

    public static java.lang.Thread ordered(int i) {
        return new java.lang.Thread(() -> {
            trace = (trace * 10) + i;
        }, name());
    }

    public static java.lang.Thread overload() {
        return new java.lang.Thread(() -> {
            trace += 2;
        });
    }
}
