public final class CarryDependencyRefusalProbe {
    public static CarryDependencyProbe receiver;
    public static void rejectedDependency(boolean flag) {
        CarryDependencyProbe.consume((receiver = CarryDependencyProbe.make()).value()
            + (CarryDependencyProbe.test(flag) ? CarryDependencyProbe.arm() : 3));
    }
}
