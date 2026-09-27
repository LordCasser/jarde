package demo;

import java.lang.reflect.Constructor;
import java.util.HashSet;
import java.util.Set;

/* JADX INFO: loaded from: original.jar:demo/Runner.class */
public final class Runner {
    public static void main(String[] args) {
        Set<Integer> parameters = new HashSet<>();
        for (Constructor<?> constructor : Numbers.class.getDeclaredConstructors()) {
            parameters.add(Integer.valueOf(constructor.getParameterTypes().length));
        }
        Set<Integer> expected = new HashSet<>();
        expected.add(2);
        expected.add(3);
        if (Numbers.values().length != 2 || Numbers.ZERO.getN() != 0 || Numbers.ONE.getN() != 1 || !parameters.equals(expected) || Numbers.ZERO.getDeclaringClass() != Numbers.class) {
            throw new AssertionError("TestEnums6 behavior differs");
        }
        System.out.println("values=ZERO:" + Numbers.ZERO.getN() + ",ONE:" + Numbers.ONE.getN());
        System.out.println("declared-constructors=" + parameters);
    }
}
