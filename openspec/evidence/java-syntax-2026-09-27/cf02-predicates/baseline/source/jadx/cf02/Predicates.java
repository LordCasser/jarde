package cf02;

/* JADX INFO: loaded from: fixture.jar:cf02/Predicates.class */
public class Predicates {
    public boolean greater(float f, float f2) {
        return f > f2;
    }

    public boolean mixed(float f, double d) {
        return ((double) f) < d;
    }

    public boolean inBounds(int[] iArr, int i) {
        return i >= 0 && i < iArr.length;
    }

    public boolean named(Object obj) {
        return obj != null && (obj instanceof String) && ((String) obj).length() > 0;
    }
}
