package demo;

/* JADX INFO: loaded from: original.jar:demo/Runner.class */
public final class Runner {
    public static void main(String[] args) {
        if (!"*".equals(DoubleOperations.TIMES.getOp()) || !"/".equals(DoubleOperations.DIVIDE.getOp()) || DoubleOperations.TIMES.apply(2.0d, 3.0d) != 6.0d || DoubleOperations.DIVIDE.apply(10.0d, 5.0d) != 2.0d || DoubleOperations.TIMES.getDeclaringClass() != DoubleOperations.class || DoubleOperations.DIVIDE.getDeclaringClass() != DoubleOperations.class || DoubleOperations.TIMES.getClass() == DoubleOperations.class || DoubleOperations.DIVIDE.getClass() == DoubleOperations.class) {
            throw new AssertionError("TestEnums2a behavior differs");
        }
        System.out.println("TIMES=*:6:" + DoubleOperations.TIMES.getClass().getName());
        System.out.println("DIVIDE=/:2:" + DoubleOperations.DIVIDE.getClass().getName());
    }
}
