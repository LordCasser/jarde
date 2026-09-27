package demo;

import java.util.HashSet;
import java.util.Set;

public final class Runner {
    public static void main(String[] args) {
        Set<Integer> parameters = new HashSet<>();
        for (java.lang.reflect.Constructor<?> constructor : Numbers.class.getDeclaredConstructors()) {
            parameters.add(constructor.getParameterTypes().length);
        }
        Set<Integer> expected = new HashSet<>();
        expected.add(2);
        expected.add(3);
        if (Numbers.values().length != 2
                || Numbers.ZERO.getN() != 0
                || Numbers.ONE.getN() != 1
                || !parameters.equals(expected)
                || Numbers.ZERO.getDeclaringClass() != Numbers.class) {
            throw new AssertionError("TestEnums6 behavior differs");
        }
        System.out.println("values=ZERO:" + Numbers.ZERO.getN() + ",ONE:" + Numbers.ONE.getN());
        System.out.println("declared-constructors=" + parameters);
    }
}
