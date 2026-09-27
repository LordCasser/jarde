package demo;

import java.lang.reflect.Constructor;
import java.util.HashSet;

/* JADX INFO: loaded from: original.jar:demo/Runner.class */
public final class Runner {
    public static void main(String[] strArr) {
        HashSet hashSet = new HashSet();
        for (Constructor<?> constructor : Numbers.class.getDeclaredConstructors()) {
            hashSet.add(Integer.valueOf(constructor.getParameterTypes().length));
        }
        HashSet hashSet2 = new HashSet();
        hashSet2.add(2);
        hashSet2.add(3);
        if (Numbers.values().length != 2 || Numbers.ZERO.getN() != 0 || Numbers.ONE.getN() != 1 || !hashSet.equals(hashSet2) || Numbers.ZERO.getDeclaringClass() != Numbers.class) {
            throw new AssertionError("TestEnums6 behavior differs");
        }
        System.out.println("values=ZERO:" + Numbers.ZERO.getN() + ",ONE:" + Numbers.ONE.getN());
        System.out.println("declared-constructors=" + hashSet);
    }
}
