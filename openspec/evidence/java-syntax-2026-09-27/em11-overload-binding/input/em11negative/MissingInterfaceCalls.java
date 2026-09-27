package em11negative;

public class MissingInterfaceCalls {
    public static String take(MissingIface value) { return "interface"; }
    public static String take(InterfaceChild value) { return "child"; }

    public static String run() {
        return take((MissingIface) new InterfaceChild());
    }
}
