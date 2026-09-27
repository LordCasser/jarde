package demo;

public final class Runner {
    public static void main(String[] args) {
        if (!"*".equals(DoubleOperations.TIMES.getOp())
                || !"/".equals(DoubleOperations.DIVIDE.getOp())
                || DoubleOperations.TIMES.apply(2, 3) != 6
                || DoubleOperations.DIVIDE.apply(10, 5) != 2
                || DoubleOperations.TIMES.getDeclaringClass() != DoubleOperations.class
                || DoubleOperations.DIVIDE.getDeclaringClass() != DoubleOperations.class
                || DoubleOperations.TIMES.getClass() == DoubleOperations.class
                || DoubleOperations.DIVIDE.getClass() == DoubleOperations.class) {
            throw new AssertionError("TestEnums2a behavior differs");
        }
        System.out.println("TIMES=*:6:" + DoubleOperations.TIMES.getClass().getName());
        System.out.println("DIVIDE=/:2:" + DoubleOperations.DIVIDE.getClass().getName());
    }
}
